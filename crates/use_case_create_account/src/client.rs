use crate::domain::AccountNameError;
use crate::domain::Error;
use crate::domain::Input;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
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
use std::ops::Deref;
use std::sync::Arc;
use utility::cache::ResourceName;
use utility::cache::TraitOperationClientError;
use utility::cache::TraitOperationClientInput;
use utility::cache::TraitOperationClientOk;
use utility::cache::TypeOperationClientInput;
use utility::cache::TypeOperationClientResult;
use utility::cache::new_resource_name;
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
    fn selected_company(&self) -> Option<CompanyUuid>;
    fn list_of_companies(&self) -> Vec<(CompanyUuid, String)>;
}

pub trait LocalModel {
    fn show_dialog(&self) -> impl HashimSignal<Dialog>;
    fn is_debit(&self) -> impl HashimSignal<bool>;
    fn is_permanent_account(&self) -> impl HashimSignal<bool>;
    fn account_name(&self) -> impl HashimSignal<String>;
    fn unit_of_measurement_of_quantity(&self) -> impl HashimSignal<String>;
    fn selected_company_name(&self) -> impl HashimSignal<String>;
    fn async_state(&self) -> impl HashimSignal<AsyncState>;
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum CompanyNameError {
    NotSelected,
    NotExist,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub enum AsyncState {
    #[default]
    Idle,
    Loading {
        input: AsyncInput,
    },
    Success {
        input: AsyncInput,
        ok: Ok,
    },
    Failure {
        input: AsyncInput,
        error: Error,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AsyncInput {
    pub user_uuid: UserUuid,
    pub is_debit: bool,
    pub is_permanent_account: bool,
    pub account_name: String,
    pub unit_of_measurement_of_quantity: String,
    pub belong_to_company: CompanyUuid,
}

pub fn is_loading(local_model: &impl LocalModel) -> bool {
    matches!(local_model.async_state().read(), AsyncState::Loading { .. })
}

pub fn error_account_name(local_model: &impl LocalModel) -> Option<AccountNameError> {
    match local_model.async_state().read() {
        AsyncState::Failure { error, .. } => error.account_name,
        _ => None,
    }
}

pub fn resolved_company_uuid(
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Option<CompanyUuid> {
    let typed = local_model.selected_company_name().read();
    if typed.is_empty() {
        return global_model.selected_company();
    }
    global_model
        .list_of_companies()
        .into_iter()
        .find_map(|(uuid, name)| (name == typed).then_some(uuid))
}

pub fn error_company_name(
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Option<CompanyNameError> {
    if let AsyncState::Failure { error, .. } = local_model.async_state().read()
        && let Some(e) = error.belong_to_company
    {
        return match e {
            RowIdError::Invalid | RowIdError::NotExist => Some(CompanyNameError::NotExist),
            RowIdError::Duplicated => None,
        };
    }

    let typed = local_model.selected_company_name().read();
    if typed.is_empty() {
        if global_model.selected_company().is_none() {
            return Some(CompanyNameError::NotSelected);
        }
        return None;
    }
    if resolved_company_uuid(local_model, global_model).is_some() {
        None
    } else {
        Some(CompanyNameError::NotExist)
    }
}

pub fn list_of_companies_to_display(
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Vec<(CompanyUuid, String)> {
    let typed = local_model.selected_company_name().read();
    let list = global_model.list_of_companies();
    if typed.is_empty() {
        return list;
    }
    select_strings(list, typed, |a| a.1.as_str())
}

#[derive(Debug, Clone)]
pub enum Change {
    ShowDialog(Dialog),
    IsDebit(bool),
    IsPermanentAccount(bool),
    AccountName(String),
    UnitOfMeasurementOfQuantity(String),
    SelectedCompanyName(String),
    AsyncState(AsyncState),
}

#[derive(Debug, Clone)]
pub enum Effect {
    Submit {
        process_id: ProcessId,
        async_input: AsyncInput,
    },
    Consent {
        process_id: ProcessId,
        user_consent: UserConsent,
    },
}

#[derive(Debug, Clone)]
pub enum Intent {
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
    Result(AsyncState),
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
            Intent::Submit => {
                if is_loading(local_model) {
                    return Ok((vec![], vec![]));
                }

                let Some(user_uuid) = global_model.user_uuid() else {
                    return Ok((vec![], vec![]));
                };

                let Some(belong_to_company) = resolved_company_uuid(local_model, global_model)
                else {
                    return Ok((vec![], vec![]));
                };

                let async_input = AsyncInput {
                    user_uuid,
                    is_debit: local_model.is_debit().read(),
                    is_permanent_account: local_model.is_permanent_account().read(),
                    account_name: local_model.account_name().read(),
                    unit_of_measurement_of_quantity: local_model
                        .unit_of_measurement_of_quantity()
                        .read(),
                    belong_to_company,
                };

                let change = vec![
                    Change::AsyncState(AsyncState::Loading {
                        input: async_input.clone(),
                    }),
                    Change::ShowDialog(Dialog::Hide),
                ];

                let effect = vec![Effect::Submit {
                    process_id,
                    async_input,
                }];

                Ok((change, effect))
            }
            Intent::Consent(v) => {
                let change = match v {
                    UserConsent::CancelOperation => {
                        vec![
                            Change::ShowDialog(Dialog::Hide),
                            Change::AsyncState(AsyncState::Idle),
                        ]
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
                    Change::ShowDialog(Default::default()),
                    Change::IsDebit(Default::default()),
                    Change::IsPermanentAccount(Default::default()),
                    Change::AccountName(Default::default()),
                    Change::UnitOfMeasurementOfQuantity(Default::default()),
                    Change::SelectedCompanyName(Default::default()),
                    Change::AsyncState(AsyncState::Idle),
                ];
                Ok((change, vec![]))
            }
            Intent::IsDebit(v) => Ok((vec![Change::IsDebit(v)], vec![])),
            Intent::IsPermanentAccount(v) => Ok((vec![Change::IsPermanentAccount(v)], vec![])),
            Intent::AccountName(v) => Ok((vec![Change::AccountName(v)], vec![])),
            Intent::UnitOfMeasurementOfQuantity(v) => {
                Ok((vec![Change::UnitOfMeasurementOfQuantity(v)], vec![]))
            }
            Intent::CompanyName(v) => Ok((vec![Change::SelectedCompanyName(v)], vec![])),
            Intent::SelectedCompany(idx) => {
                let list = list_of_companies_to_display(local_model, global_model);
                match list.get(idx) {
                    Some(a) => {
                        let change = vec![Change::SelectedCompanyName(a.1.clone())];
                        Ok((change, vec![]))
                    }
                    None => Ok((vec![], vec![])),
                }
            }
        },
        Message::Observe(observe) => match observe {
            Observe::ShowDialog => Ok((vec![Change::ShowDialog(Dialog::Show)], vec![])),
            Observe::HideDialog => Ok((vec![Change::ShowDialog(Dialog::Hide)], vec![])),
            Observe::Result(result) => {
                let change = match result {
                    AsyncState::Success { .. } => {
                        vec![
                            Change::ShowDialog(Default::default()),
                            Change::AccountName(Default::default()),
                            Change::IsDebit(Default::default()),
                            Change::IsPermanentAccount(Default::default()),
                            Change::UnitOfMeasurementOfQuantity(Default::default()),
                            Change::SelectedCompanyName(Default::default()),
                            Change::AsyncState(result),
                        ]
                    }
                    _ => vec![
                        Change::ShowDialog(Default::default()),
                        Change::AsyncState(result),
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
        Change::IsDebit(i) => local_model.is_debit().set(i),
        Change::IsPermanentAccount(i) => local_model.is_permanent_account().set(i),
        Change::AccountName(i) => local_model.account_name().set(i),
        Change::UnitOfMeasurementOfQuantity(i) => {
            local_model.unit_of_measurement_of_quantity().set(i)
        }
        Change::SelectedCompanyName(i) => local_model.selected_company_name().set(i),
        Change::AsyncState(i) => local_model.async_state().set(i),
    }
}

pub async fn effect(msg: Effect, mut context: UiContext) -> Result<()> {
    match msg {
        Effect::Submit {
            process_id,
            async_input,
        } => {
            handle_submit(process_id, async_input, context).await?;
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

async fn handle_submit(process_id: ProcessId, input: AsyncInput, context: UiContext) -> Result<()> {
    let context1 = context.clone();

    let dialog_signal_adapter = Arc::new(DialogDispatchAdapter {
        process_id,
        sender: context.sender_to_commander.clone(),
    });

    let new_uuid = AccountUuid::from(UuidType::from(Id::generate()));

    let data: TypeOperationClientInput = Arc::new(Input {
        user_uuid: input.user_uuid.clone(),
        new_uuid,
        is_debit: input.is_debit,
        is_permanent_account: input.is_permanent_account,
        account_name: input.account_name.clone(),
        unit_of_measurement_of_quantity: input.unit_of_measurement_of_quantity.clone(),
        belong_to_company: input.belong_to_company.clone(),
    });

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

            let async_state = match result {
                Ok(a) => AsyncState::Success {
                    input: input.clone(),
                    ok: a,
                },
                Err(a) => AsyncState::Failure {
                    input: input.clone(),
                    error: a,
                },
            };

            context1
                .sender_to_commander
                .send(process_id, Message::Observe(Observe::Result(async_state)));

            Ok(is_ok)
        },
    )
    .await?;

    Ok(())
}
