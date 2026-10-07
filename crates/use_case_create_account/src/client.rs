use crate::domain::AccountNameError;
use crate::domain::Error;
use crate::domain::Input;
use crate::domain::MyResult;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use infrastructure::actors::Receiver;
use infrastructure::actors::Sender;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use kernel::client::Cache;
use kernel::new_types::AccountUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::new_types::UuidType;
use kernel::types::DatabaseRead;
use kernel::types::MyErrorTrait;
use kernel::types::RowIdError;
use serde::Deserialize;
use serde::Serialize;
use std::any::Any;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use use_case_get_all_accounts::client::fetch;
use utility::cache::CachingStrategy;
use utility::cache::ResourceName;
use utility::cache::Response;
use utility::cache::TraitOperationClientError;
use utility::cache::TraitOperationClientInput;
use utility::cache::TraitOperationClientOk;
use utility::cache::TypeOperationClientInput;
use utility::cache::TypeOperationClientResult;
use utility::cache::new_resource_name;
use utility::dtos::TxnNumber;
use utility::process_manager::MessageToProcessManager;
use utility::process_manager::ProcessDialog;
use utility::process_manager::ProcessId;
use utility::process_manager::UserConsent;
use utility::tools::select_strings;
use utility::ui_effect::Commander;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UiContext;
use utility::ui_orchestration::handle_fall_back;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[new_resource_name("accounts")];

impl TraitOperationClientOk for Ok {
    fn subs_to_poke(&self) -> &'static [ResourceName] {
        RESOURCES_NAME_TO_POKE
    }
}

impl TraitOperationClientError for Error {
    fn subs_to_poke(&self) -> &'static [ResourceName] {
        RESOURCES_NAME_TO_POKE
    }
}

impl TraitOperationClientInput for Input {
    fn user_uuid(&self) -> Option<[u8; 16]> {
        Some(*self.user_uuid.deref().deref())
    }
}

pub async fn check_input<
    Ch: Cache,
    DBReader: for<'a> DatabaseRead<Db<'a> = Ch, Input = ReadInput, Output = ReadOutput>,
>(
    input: &Input,
    cache: &mut Ch,
) -> Result<TypeOperationClientResult> {
    let errr = input.state_full_check::<DBReader>(cache).await?;

    if errr.is_there_error() {
        return Ok(Err(Box::new(errr)));
    }

    let state_less_operation = input.state_less_operation();
    Ok(Ok(Arc::new(state_less_operation)))
}

pub trait GlobalModel {
    fn user_uuid(&self) -> Option<UserUuid>;
    fn selected_company(&self) -> impl HashimSignal<Option<CompanyUuid>>;
    fn list_of_companies(&self) -> Vec<(CompanyUuid, String)>;
}

pub trait LocalModel {
    fn show_dialog(&self) -> impl HashimSignal<Dialog>;
    fn is_loading(&self) -> impl HashimSignal<bool>;
    fn is_debit(&self) -> impl HashimSignal<bool>;
    fn is_permanent_account(&self) -> impl HashimSignal<bool>;
    fn account_name(&self) -> impl HashimSignal<String>;
    fn unit_of_measurement_of_quantity(&self) -> impl HashimSignal<String>;
    fn account_name_error(&self) -> impl HashimSignal<Option<AccountNameError>>;
    fn selected_company_name(&self) -> impl HashimSignal<String>;
    fn selected_company_uuid(&self) -> impl HashimSignal<Option<CompanyUuid>>;
    fn list_of_companies_to_display(&self) -> impl HashimSignal<Vec<(CompanyUuid, String)>>;
    fn company_name_error(&self) -> impl HashimSignal<Option<CompanyNameError>>;
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum CompanyNameError {
    NotSelected,
    NotExist,
    Invalid,
}

impl From<RowIdError> for CompanyNameError {
    fn from(e: RowIdError) -> Self {
        match e {
            RowIdError::Invalid => CompanyNameError::Invalid,
            RowIdError::NotExist => CompanyNameError::NotExist,
            RowIdError::Duplicated => CompanyNameError::NotExist,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Change {
    ShowDialog(Dialog),
    IsLoading(bool),
    IsDebit(bool),
    IsPermanentAccount(bool),
    AccountName(String),
    UnitOfMeasurementOfQuantity(String),
    AccountNameError(Option<AccountNameError>),
    SelectedCompanyName(String),
    SelectedCompanyUuid(Option<CompanyUuid>),
    ListOfCompaniesToDisplay(Vec<(CompanyUuid, String)>),
    CompanyNameError(Option<CompanyNameError>),
}

#[derive(Debug, Clone)]
pub enum Effect {
    Submit {
        process_id: ProcessId,
        user_uuid: UserUuid,
        is_debit: bool,
        is_permanent_account: bool,
        account_name: String,
        unit_of_measurement_of_quantity: String,
        belong_to_company: CompanyUuid,
    },
    Consent {
        process_id: ProcessId,
        user_consent: UserConsent,
    },
    Check {
        process_id: ProcessId,
        is_debit: bool,
        is_permanent_account: bool,
        account_name: String,
        unit_of_measurement_of_quantity: String,
        user_uuid: UserUuid,
        belong_to_company: CompanyUuid,
    },
    Refresh {
        user_uuid: UserUuid,
        company_uuid: CompanyUuid,
    },
}

#[derive(Debug, Clone)]
pub enum Intent {
    Subscribe,
    Submit,
    Consent(UserConsent),
    Clean,
    IsDebit(bool),
    IsPermanentAccount(bool),
    AccountName(String),
    UnitOfMeasurementOfQuantity(String),
    CompanyName(String),
    SelectedCompany(usize),
}

#[derive(Debug, Clone)]
pub enum Observe {
    ShowDialog,
    HideDialog,
    SubmitResult(MyResult),
    CheckResult(MyResult),
}

#[derive(Debug, Clone)]
pub enum Message {
    Intent(Intent),
    Observe(Observe),
}

impl MessageTrait for Message {}

pub fn reduce(
    msg: Message,
    process_id: ProcessId,
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Result<(Vec<Change>, Vec<Effect>)> {
    match msg {
        Message::Intent(intent) => match intent {
            Intent::Subscribe => {
                let Some(user_uuid) = global_model.user_uuid() else {
                    return Ok((vec![], vec![]));
                };
                let Some(company_uuid) = global_model.selected_company().read() else {
                    return Ok((vec![], vec![]));
                };
                let effect = vec![Effect::Refresh {
                    user_uuid,
                    company_uuid,
                }];
                Ok((vec![], effect))
            }
            Intent::Submit => {
                let Some(user_uuid) = global_model.user_uuid() else {
                    return Ok((vec![], vec![]));
                };

                let typed = local_model.selected_company_name().read();
                let local_pick = local_model.selected_company_uuid().read();

                let belong_to_company = local_pick.or_else(|| {
                    if typed.is_empty() {
                        global_model.selected_company().read()
                    } else {
                        None
                    }
                });

                let Some(belong_to_company) = belong_to_company else {
                    let err = if typed.is_empty() {
                        CompanyNameError::NotSelected
                    } else {
                        CompanyNameError::NotExist
                    };
                    return Ok((vec![Change::CompanyNameError(Some(err))], vec![]));
                };

                let change = vec![
                    Change::ShowDialog(Default::default()),
                    Change::AccountNameError(None),
                    Change::CompanyNameError(None),
                ];
                let effect = vec![Effect::Submit {
                    process_id,
                    user_uuid,
                    is_debit: local_model.is_debit().read(),
                    is_permanent_account: local_model.is_permanent_account().read(),
                    account_name: local_model.account_name().read(),
                    unit_of_measurement_of_quantity: local_model
                        .unit_of_measurement_of_quantity()
                        .read(),
                    belong_to_company,
                }];
                Ok((change, effect))
            }
            Intent::Consent(v) => {
                let change = match v {
                    UserConsent::CancelOperation => {
                        vec![Change::ShowDialog(Dialog::Hide), Change::IsLoading(false)]
                    }
                    _ => vec![Change::ShowDialog(Dialog::Hide)],
                };
                let effect = vec![Effect::Consent {
                    process_id,
                    user_consent: v,
                }];
                Ok((change, effect))
            }
            Intent::Clean => {
                let change = vec![
                    Change::AccountName(Default::default()),
                    Change::IsDebit(Default::default()),
                    Change::IsPermanentAccount(Default::default()),
                    Change::UnitOfMeasurementOfQuantity(Default::default()),
                    Change::IsLoading(Default::default()),
                    Change::AccountNameError(Default::default()),
                    Change::SelectedCompanyName(Default::default()),
                    Change::SelectedCompanyUuid(Default::default()),
                    Change::ListOfCompaniesToDisplay(Default::default()),
                    Change::CompanyNameError(Default::default()),
                ];
                Ok((change, vec![]))
            }
            Intent::IsDebit(v) => Ok((vec![Change::IsDebit(v)], vec![])),
            Intent::IsPermanentAccount(v) => Ok((vec![Change::IsPermanentAccount(v)], vec![])),
            Intent::AccountName(v) => {
                let Some(user_uuid) = global_model.user_uuid() else {
                    return Ok((vec![Change::AccountName(v)], vec![]));
                };
                let Some(belong_to_company) = local_model
                    .selected_company_uuid()
                    .read()
                    .or_else(|| global_model.selected_company().read())
                else {
                    return Ok((vec![Change::AccountName(v)], vec![]));
                };

                let change = vec![Change::AccountName(v.clone())];
                let effect = vec![Effect::Check {
                    process_id,
                    is_debit: local_model.is_debit().read(),
                    is_permanent_account: local_model.is_permanent_account().read(),
                    account_name: v,
                    unit_of_measurement_of_quantity: local_model
                        .unit_of_measurement_of_quantity()
                        .read(),
                    user_uuid,
                    belong_to_company,
                }];
                Ok((change, effect))
            }
            Intent::UnitOfMeasurementOfQuantity(v) => {
                Ok((vec![Change::UnitOfMeasurementOfQuantity(v)], vec![]))
            }
            Intent::CompanyName(v) => {
                let list_of_companies = global_model.list_of_companies();
                let filtered = select_strings(list_of_companies, v.clone(), |a| a.1.as_str());

                let uuid = match filtered.get(0) {
                    Some(a) if v == a.1 => Some(a.0.clone()),
                    _ => None,
                };

                let change = vec![
                    Change::SelectedCompanyName(v),
                    Change::ListOfCompaniesToDisplay(filtered),
                    Change::SelectedCompanyUuid(uuid),
                    Change::CompanyNameError(None),
                ];
                Ok((change, vec![]))
            }
            Intent::SelectedCompany(idx) => {
                let list = local_model.list_of_companies_to_display().read();
                match list.get(idx) {
                    Some(a) => {
                        let change = vec![
                            Change::SelectedCompanyName(a.1.clone()),
                            Change::SelectedCompanyUuid(Some(a.0.clone())),
                            Change::CompanyNameError(None),
                        ];
                        Ok((change, vec![]))
                    }
                    None => Ok((vec![], vec![])),
                }
            }
        },
        Message::Observe(observe) => match observe {
            Observe::ShowDialog => Ok((vec![Change::ShowDialog(Dialog::Show)], vec![])),
            Observe::HideDialog => Ok((vec![Change::ShowDialog(Dialog::Hide)], vec![])),
            Observe::SubmitResult(result) => {
                let change = match result {
                    Ok(_) => vec![
                        Change::ShowDialog(Default::default()),
                        Change::IsLoading(Default::default()),
                        Change::AccountName(Default::default()),
                        Change::IsDebit(Default::default()),
                        Change::IsPermanentAccount(Default::default()),
                        Change::UnitOfMeasurementOfQuantity(Default::default()),
                        Change::AccountNameError(None),
                        Change::CompanyNameError(None),
                    ],
                    Err(a) => vec![
                        Change::ShowDialog(Default::default()),
                        Change::IsLoading(Default::default()),
                        Change::AccountNameError(a.account_name),
                        Change::CompanyNameError(a.belong_to_company.map(CompanyNameError::from)),
                    ],
                };
                Ok((change, vec![]))
            }
            Observe::CheckResult(result) => {
                let change = match result {
                    Ok(_) => vec![
                        Change::AccountNameError(None),
                        Change::CompanyNameError(None),
                    ],
                    Err(a) => vec![
                        Change::AccountNameError(a.account_name),
                        Change::CompanyNameError(a.belong_to_company.map(CompanyNameError::from)),
                    ],
                };
                Ok((change, vec![]))
            }
        },
    }
}

pub fn update(msg: Change, local_model: &impl LocalModel, _global_model: &impl GlobalModel) {
    match msg {
        Change::ShowDialog(i) => local_model.show_dialog().set(i),
        Change::IsLoading(i) => local_model.is_loading().set(i),
        Change::IsDebit(i) => local_model.is_debit().set(i),
        Change::IsPermanentAccount(i) => local_model.is_permanent_account().set(i),
        Change::AccountName(i) => local_model.account_name().set(i),
        Change::UnitOfMeasurementOfQuantity(i) => {
            local_model.unit_of_measurement_of_quantity().set(i)
        }
        Change::AccountNameError(i) => local_model.account_name_error().set(i),
        Change::SelectedCompanyName(i) => local_model.selected_company_name().set(i),
        Change::SelectedCompanyUuid(i) => local_model.selected_company_uuid().set(i),
        Change::ListOfCompaniesToDisplay(i) => local_model.list_of_companies_to_display().set(i),
        Change::CompanyNameError(i) => local_model.company_name_error().set(i),
    }
}

pub async fn effect(msg: Effect, mut context: UiContext) -> Result<()> {
    match msg {
        Effect::Submit {
            process_id,
            user_uuid,
            is_debit,
            is_permanent_account,
            account_name,
            unit_of_measurement_of_quantity,
            belong_to_company,
        } => {
            let new_uuid = AccountUuid::from(UuidType::from(Id::generate()));

            let input = Input {
                user_uuid,
                new_uuid,
                is_debit,
                is_permanent_account,
                account_name,
                unit_of_measurement_of_quantity,
                belong_to_company,
            };
            handle_submit(process_id, &input, context).await?;
        }
        Effect::Consent {
            process_id,
            user_consent,
        } => {
            context
                .sender_to_process_manager
                .send(MessageToProcessManager::FromUser {
                    process_id,
                    consent: user_consent,
                })
                .await?;
        }
        Effect::Check {
            process_id,
            is_debit,
            is_permanent_account,
            account_name,
            unit_of_measurement_of_quantity,
            user_uuid,
            belong_to_company,
        } => {
            let new_uuid = AccountUuid::from(UuidType::from(Id::generate()));

            let input = Input {
                user_uuid,
                new_uuid,
                is_debit,
                is_permanent_account,
                account_name,
                unit_of_measurement_of_quantity,
                belong_to_company,
            };
            handle_check(process_id, input, context).await?;
        }
        Effect::Refresh {
            user_uuid,
            company_uuid,
        } => {
            fetch(company_uuid, user_uuid, context.cache).await?;
        }
    }
    Ok(())
}

struct DialogDispatchAdapter {
    process_id: ProcessId,
    sender: Commander,
}

impl ProcessDialog for DialogDispatchAdapter {
    fn show(&self) {
        self.sender
            .send(self.process_id, Message::Observe(Observe::ShowDialog));
    }

    fn hide(&self) {
        self.sender
            .send(self.process_id, Message::Observe(Observe::HideDialog));
    }
}

async fn handle_submit(process_id: ProcessId, input: &Input, context: UiContext) -> Result<()> {
    let context1 = context.clone();

    let dialog_signal_adapter = Arc::new(DialogDispatchAdapter {
        process_id,
        sender: context.sender_to_commander.clone(),
    });

    let data: TypeOperationClientInput = Arc::new(input.clone());

    handle_fall_back(
        context.sender_to_error,
        context.cache,
        context.sender_to_process_manager,
        dialog_signal_adapter,
        process_id,
        data,
        move |data| {
            let result = match data {
                Ok(ok) => {
                    let a: Arc<dyn Any> = ok;
                    let a: &Ok = a.downcast_ref().context("downcast error")?;
                    Ok(a.clone())
                }
                Err(err) => {
                    let a: Box<dyn Any> = err;
                    let a: Box<Error> = a.downcast().map_err(|_| anyhow!("downcast error"))?;
                    Err(*a)
                }
            };

            let is_ok = result.is_ok();

            context1
                .sender_to_commander
                .send(process_id, Message::Observe(Observe::SubmitResult(result)));

            Ok(is_ok)
        },
    )
    .await?;

    Ok(())
}

async fn handle_check(process_id: ProcessId, input: Input, mut context: UiContext) -> Result<()> {
    let data: TypeOperationClientInput = Arc::new(input);

    let mut receiver_to_response = context
        .cache
        .send_to_cache_actor(CachingStrategy::ReadCacheOnly, TxnNumber::default(), data)
        .await?;

    match receiver_to_response.recv().await? {
        Response::CloseTheChannel => {}
        Response::ServerCannotBeReached => {}
        Response::Data {
            is_response_from_server: _,
            data,
        } => {
            let result = match data {
                Ok(ok) => {
                    let a: Arc<dyn Any> = ok;
                    let a: &Ok = a.downcast_ref().context("downcast error")?;
                    Ok(a.clone())
                }
                Err(err) => {
                    let a: Box<dyn Any> = err;
                    let a: Box<Error> = a.downcast().map_err(|_| anyhow!("downcast error"))?;
                    Err(*a)
                }
            };

            context
                .sender_to_commander
                .send(process_id, Message::Observe(Observe::CheckResult(result)));
        }
    }

    Ok(())
}
