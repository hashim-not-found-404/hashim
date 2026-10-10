use crate::domain::AccountNameError;
use crate::domain::Error;
use crate::domain::Input;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
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
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use utility::cache::CachingStrategy;
use utility::cache::ResourceName;
use utility::cache::Response;
use utility::cache::ResponseFunction;
use utility::cache::TraitOperationClientError;
use utility::cache::TraitOperationClientInput;
use utility::cache::TraitOperationClientOk;
use utility::cache::TypeOperationClientError;
use utility::cache::TypeOperationClientInput;
use utility::cache::TypeOperationClientResult;
use utility::cache::new_resource_name;
use utility::dtos::TxnNumber;
use utility::process_manager::ProcessId;
use utility::process_manager::UserConsent;
use utility::tools::select_strings;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UiContext;
use utility::ui_orchestration::GenricAsyncState;
use utility::ui_orchestration::spawn_listener;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[new_resource_name("accounts")];
const RESOURCES_NAME_TO_LISTEN: &[ResourceName] = &[new_resource_name("accounts")];

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

    fn state_less_check(&self) -> Option<TypeOperationClientError> {
        let errr = self.state_less_check();
        if errr.is_there_error() {
            Some(Box::new(errr))
        } else {
            None
        }
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

pub(crate) type AsyncState = GenricAsyncState<AsyncInput, Ok, Error>;

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AsyncInput {
    pub user_uuid: UserUuid,
    pub is_debit: bool,
    pub is_permanent_account: bool,
    pub account_name: String,
    pub unit_of_measurement_of_quantity: String,
    pub belong_to_company: CompanyUuid,
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
    SpawnTimer,
    Submit {
        async_input: AsyncInput,
        is_to_server: bool,
    },
    Check {
        async_input: AsyncInput,
    },
    Subscribe,
    UnSubscribe,
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
    Subscribe,
    UnSubscribe,
}

#[derive(Debug, Clone)]
pub enum Observe {
    Timeout,
    Result(AsyncState),
    Refresh,
}

#[derive(Debug, Clone)]
pub enum Message {
    Intent(Intent),
    Observe(Observe),
}

impl MessageTrait for Message {}

fn build_check_effect(
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
    override_company: Option<CompanyUuid>,
    mutate: impl FnOnce(&mut AsyncInput),
) -> Vec<Effect> {
    let Some(user_uuid) = global_model.user_uuid() else {
        return vec![];
    };
    let belong_to_company =
        override_company.or_else(|| resolved_company_uuid(local_model, global_model));
    let Some(belong_to_company) = belong_to_company else {
        return vec![];
    };
    let mut a = AsyncInput {
        user_uuid,
        is_debit: local_model.is_debit().read(),
        is_permanent_account: local_model.is_permanent_account().read(),
        account_name: local_model.account_name().read(),
        unit_of_measurement_of_quantity: local_model.unit_of_measurement_of_quantity().read(),
        belong_to_company,
    };
    mutate(&mut a);
    vec![Effect::Check { async_input: a }]
}

pub fn reduce(
    msg: Message,
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> (Vec<Change>, Vec<Effect>) {
    if let Message::Intent(ref a) = msg {
        match a {
            Intent::Consent(_) | Intent::Subscribe | Intent::UnSubscribe => {}
            _ => {
                if local_model.async_state().read().is_loading() {
                    return (vec![], vec![]);
                }
            }
        }
    }

    match msg {
        Message::Intent(intent) => match intent {
            Intent::Submit => {
                let Some(user_uuid) = global_model.user_uuid() else {
                    return (vec![], vec![]);
                };

                let Some(belong_to_company) = resolved_company_uuid(local_model, global_model)
                else {
                    return (vec![], vec![]);
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

                let effect = vec![
                    Effect::SpawnTimer,
                    Effect::Submit {
                        async_input,
                        is_to_server: true,
                    },
                ];

                (change, effect)
            }
            Intent::Consent(v) => {
                let (change, effect) = match v {
                    UserConsent::CancelOperation => (
                        vec![
                            Change::ShowDialog(Dialog::Hide),
                            Change::AsyncState(AsyncState::Idle),
                        ],
                        vec![],
                    ),
                    UserConsent::WaitForServerResponse => (
                        vec![Change::ShowDialog(Dialog::Hide)],
                        vec![Effect::SpawnTimer],
                    ),
                    UserConsent::DontWaitForServerResponse => {
                        let Some(user_uuid) = global_model.user_uuid() else {
                            return (vec![], vec![]);
                        };
                        let Some(belong_to_company) =
                            resolved_company_uuid(local_model, global_model)
                        else {
                            return (vec![], vec![]);
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

                        (
                            vec![Change::ShowDialog(Dialog::Hide)],
                            vec![Effect::Submit {
                                async_input,
                                is_to_server: false,
                            }],
                        )
                    }
                };

                (change, effect)
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
                (change, vec![])
            }
            Intent::IsDebit(v) => {
                let change = vec![Change::IsDebit(v)];
                let effect = build_check_effect(local_model, global_model, None, |a| {
                    a.is_debit = v;
                });
                (change, effect)
            }
            Intent::IsPermanentAccount(v) => {
                let change = vec![Change::IsPermanentAccount(v)];
                let effect = build_check_effect(local_model, global_model, None, |a| {
                    a.is_permanent_account = v;
                });
                (change, effect)
            }
            Intent::AccountName(v) => {
                let change = vec![Change::AccountName(v.clone())];
                let effect = build_check_effect(local_model, global_model, None, |a| {
                    a.account_name = v;
                });
                (change, effect)
            }
            Intent::UnitOfMeasurementOfQuantity(v) => {
                let change = vec![Change::UnitOfMeasurementOfQuantity(v.clone())];
                let effect = build_check_effect(local_model, global_model, None, |a| {
                    a.unit_of_measurement_of_quantity = v;
                });
                (change, effect)
            }
            Intent::CompanyName(v) => {
                let change = vec![Change::SelectedCompanyName(v.clone())];
                let resolved = if v.is_empty() {
                    global_model.selected_company()
                } else {
                    global_model
                        .list_of_companies()
                        .into_iter()
                        .find_map(|(uuid, name)| (name == v).then_some(uuid))
                };
                let effect = build_check_effect(local_model, global_model, resolved, |_| {});
                (change, effect)
            }
            Intent::SelectedCompany(idx) => {
                let list = list_of_companies_to_display(local_model, global_model);
                match list.get(idx) {
                    Some((company_uuid, company_name)) => {
                        let change = vec![Change::SelectedCompanyName(company_name.clone())];
                        let effect = build_check_effect(
                            local_model,
                            global_model,
                            Some(company_uuid.clone()),
                            |_| {},
                        );
                        (change, effect)
                    }
                    None => (vec![], vec![]),
                }
            }
            Intent::Subscribe => {
                let effect = vec![Effect::Subscribe];
                (vec![], effect)
            }
            Intent::UnSubscribe => {
                let effect = vec![Effect::UnSubscribe];
                (vec![], effect)
            }
        },
        Message::Observe(observe) => match observe {
            Observe::Timeout => {
                let change = match local_model.async_state().read() {
                    AsyncState::Loading { .. } => vec![Change::ShowDialog(Dialog::Show)],
                    _ => vec![],
                };
                (change, vec![])
            }
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
                (change, vec![])
            }
            Observe::Refresh => {
                let effect = build_check_effect(local_model, global_model, None, |_| {});
                (vec![], effect)
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

pub async fn effect(msg: Effect, process_id: ProcessId, context: UiContext) -> Result<()> {
    match msg {
        Effect::SpawnTimer => {
            Rt::spawn_local(async move {
                Rt::sleep(Duration::from_secs(5)).await;
                let _ = context
                    .sender_to_commander
                    .async_send(process_id, Message::Observe(Observe::Timeout))
                    .await;
            });
        }
        Effect::Submit {
            async_input,
            is_to_server,
        } => {
            handle_submit(process_id, async_input, context, is_to_server).await?;
        }
        Effect::Check { async_input } => {
            handle_check(process_id, async_input, context).await?;
        }
        Effect::Subscribe => {
            handle_subscribe(process_id, context).await?;
        }
        Effect::UnSubscribe => {
            context.aborters.abort(process_id).await?;
        }
    }
    Ok(())
}

async fn handle_submit(
    process_id: ProcessId,
    input: AsyncInput,
    mut context: UiContext,
    is_to_server: bool,
) -> Result<()> {
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

    let f: ResponseFunction = Box::new(move |a| -> Pin<Box<dyn Future<Output = Result<()>>>> {
        Box::pin({
            let sender = context.sender_to_commander.clone();
            let input = input.clone();

            async move {
                let observe = match a {
                    Response::ServerCannotBeReached => Observe::Timeout,
                    Response::Data { data } => match data {
                        Ok(ok) => {
                            let a: Arc<dyn Any> = ok;
                            let a: &Ok = a.downcast_ref().context("downcast error")?;

                            Observe::Result(AsyncState::Success {
                                input: input.clone(),
                                ok: a.clone(),
                            })
                        }
                        Err(err) => {
                            let a: Box<dyn Any> = err;
                            let a: Box<Error> =
                                a.downcast().map_err(|_| anyhow!("downcast error"))?;

                            Observe::Result(AsyncState::Failure { input, error: *a })
                        }
                    },
                };

                sender
                    .async_send(process_id.clone(), Message::Observe(observe))
                    .await?;

                Ok(())
            }
        })
    });

    let strategy = if is_to_server {
        CachingStrategy::WriteServerOnly
    } else {
        CachingStrategy::WriteCacheOnly
    };

    context
        .cache
        .send_to_cache_actor(strategy, TxnNumber::default(), data, f)
        .await?;

    Ok(())
}

async fn handle_check(
    process_id: ProcessId,
    input: AsyncInput,
    mut context: UiContext,
) -> Result<()> {
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

    let f: ResponseFunction = Box::new(move |a| -> Pin<Box<dyn Future<Output = Result<()>>>> {
        Box::pin({
            let sender = context.sender_to_commander.clone();
            let input = input.clone();

            async move {
                if let Response::Data { data } = a {
                    let new_state = match data {
                        Ok(_) => AsyncState::Idle,
                        Err(err) => {
                            let a: Box<dyn Any> = err;
                            let a: Box<Error> =
                                a.downcast().map_err(|_| anyhow!("downcast error"))?;
                            AsyncState::Failure { input, error: *a }
                        }
                    };

                    sender
                        .async_send(
                            process_id.clone(),
                            Message::Observe(Observe::Result(new_state)),
                        )
                        .await?;
                }

                Ok(())
            }
        })
    });

    context
        .cache
        .send_to_cache_actor(
            CachingStrategy::ReadCacheOnly,
            TxnNumber::default(),
            data,
            f,
        )
        .await?;

    Ok(())
}

async fn handle_subscribe(process_id: ProcessId, context: UiContext) -> Result<()> {
    spawn_listener(
        context,
        RESOURCES_NAME_TO_LISTEN,
        process_id,
        Message::Observe(Observe::Refresh),
    )
    .await
}
