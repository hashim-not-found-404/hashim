use crate::domain::BranchNameError;
use crate::domain::Error;
use crate::domain::Input;
use crate::domain::LocationError;
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
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::new_types::UuidType;
use kernel::types::Currency;
use kernel::types::DatabaseRead;
use kernel::types::Location;
use kernel::types::MyErrorTrait;
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

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[new_resource_name("branches")];
const RESOURCES_NAME_TO_LISTEN: &[ResourceName] = &[new_resource_name("branches")];

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
    fn company_name(&self) -> impl HashimSignal<String>;
    fn branch_name(&self) -> impl HashimSignal<String>;
    fn currency(&self) -> impl HashimSignal<Currency>;
    fn location(&self) -> impl HashimSignal<Location>;
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
    pub company_belong: CompanyUuid,
    pub branch_name: String,
    pub currency: Currency,
    pub location: Location,
}

pub fn error_branch_name(local_model: &impl LocalModel) -> Option<BranchNameError> {
    match local_model.async_state().read() {
        AsyncState::Failure { error, .. } => error.branch_name,
        _ => None,
    }
}

pub fn error_location(local_model: &impl LocalModel) -> Option<LocationError> {
    match local_model.async_state().read() {
        AsyncState::Failure { error, .. } => error.location,
        _ => None,
    }
}

pub fn resolved_company_uuid(
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Option<CompanyUuid> {
    let typed = local_model.company_name().read();
    if typed.is_empty() {
        return global_model.selected_company();
    }
    global_model
        .list_of_companies()
        .into_iter()
        .find_map(|(uuid, name)| (name == typed).then_some(uuid))
}

pub fn company_name_error(
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Option<CompanyNameError> {
    let typed = local_model.company_name().read();
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
    let typed = local_model.company_name().read();
    let list = global_model.list_of_companies();
    if typed.is_empty() {
        return list;
    }
    select_strings(list, typed, |a| a.1.as_str())
}

#[derive(Debug, Clone)]
pub enum Change {
    ShowDialog(Dialog),
    CompanyName(String),
    BranchName(String),
    Currency(Currency),
    Location(Location),
    AsyncState(AsyncState),
}

#[derive(Debug, Clone)]
pub enum Effect {
    SpawnTimer {
        process_id: ProcessId,
    },
    Submit {
        process_id: ProcessId,
        async_input: AsyncInput,
        is_to_server: bool,
    },
    Check {
        process_id: ProcessId,
        async_input: AsyncInput,
    },
    Subscribe {
        process_id: ProcessId,
    },
    UnSubscribe {
        process_id: ProcessId,
    },
}

#[derive(Debug, Clone)]
pub enum Intent {
    Submit,
    Consent(UserConsent),
    Clean,
    CompanyName(String),
    SelectedCompany(usize),
    BranchName(String),
    Currency(Currency),
    Latitude(f64),
    Longitude(f64),
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
    process_id: ProcessId,
    override_company: Option<CompanyUuid>,
    mutate: impl FnOnce(&mut AsyncInput),
) -> Vec<Effect> {
    let Some(user_uuid) = global_model.user_uuid() else {
        return vec![];
    };
    let company_belong =
        override_company.or_else(|| resolved_company_uuid(local_model, global_model));
    let Some(company_belong) = company_belong else {
        return vec![];
    };
    let mut a = AsyncInput {
        user_uuid,
        company_belong,
        branch_name: local_model.branch_name().read(),
        currency: local_model.currency().read(),
        location: local_model.location().read(),
    };
    mutate(&mut a);
    vec![Effect::Check {
        process_id,
        async_input: a,
    }]
}

pub fn reduce(
    msg: Message,
    process_id: ProcessId,
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Result<(Vec<Change>, Vec<Effect>)> {
    if let Message::Intent(ref a) = msg {
        match a {
            Intent::Consent(_) | Intent::Subscribe | Intent::UnSubscribe => {}
            _ => {
                if local_model.async_state().read().is_loading() {
                    return Ok((vec![], vec![]));
                }
            }
        }
    }

    match msg {
        Message::Intent(intent) => match intent {
            Intent::Submit => {
                let Some(user_uuid) = global_model.user_uuid() else {
                    return Ok((vec![], vec![]));
                };

                let Some(company_belong) = resolved_company_uuid(local_model, global_model) else {
                    return Ok((vec![], vec![]));
                };

                let async_input = AsyncInput {
                    user_uuid,
                    company_belong,
                    branch_name: local_model.branch_name().read(),
                    currency: local_model.currency().read(),
                    location: local_model.location().read(),
                };

                let change = vec![
                    Change::AsyncState(AsyncState::Loading {
                        input: async_input.clone(),
                    }),
                    Change::ShowDialog(Dialog::Hide),
                ];

                let effect = vec![
                    Effect::SpawnTimer { process_id },
                    Effect::Submit {
                        process_id,
                        async_input,
                        is_to_server: true,
                    },
                ];

                Ok((change, effect))
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
                        vec![Effect::SpawnTimer { process_id }],
                    ),
                    UserConsent::DontWaitForServerResponse => {
                        let Some(user_uuid) = global_model.user_uuid() else {
                            return Ok((vec![], vec![]));
                        };
                        let Some(company_belong) = resolved_company_uuid(local_model, global_model)
                        else {
                            return Ok((vec![], vec![]));
                        };
                        let async_input = AsyncInput {
                            user_uuid,
                            company_belong,
                            branch_name: local_model.branch_name().read(),
                            currency: local_model.currency().read(),
                            location: local_model.location().read(),
                        };

                        (
                            vec![Change::ShowDialog(Dialog::Hide)],
                            vec![Effect::Submit {
                                process_id,
                                async_input,
                                is_to_server: false,
                            }],
                        )
                    }
                };

                Ok((change, effect))
            }
            Intent::Clean => {
                let change = vec![
                    Change::ShowDialog(Default::default()),
                    Change::CompanyName(Default::default()),
                    Change::BranchName(Default::default()),
                    Change::Currency(Default::default()),
                    Change::Location(Default::default()),
                    Change::AsyncState(AsyncState::Idle),
                ];
                Ok((change, vec![]))
            }
            Intent::CompanyName(v) => {
                let change = vec![Change::CompanyName(v.clone())];
                let resolved = if v.is_empty() {
                    global_model.selected_company()
                } else {
                    global_model
                        .list_of_companies()
                        .into_iter()
                        .find_map(|(uuid, name)| (name == v).then_some(uuid))
                };
                let effect =
                    build_check_effect(local_model, global_model, process_id, resolved, |_| {});
                Ok((change, effect))
            }
            Intent::SelectedCompany(idx) => {
                let list = list_of_companies_to_display(local_model, global_model);
                match list.get(idx) {
                    Some((company_uuid, company_name)) => {
                        let change = vec![Change::CompanyName(company_name.clone())];
                        let effect = build_check_effect(
                            local_model,
                            global_model,
                            process_id,
                            Some(company_uuid.clone()),
                            |_| {},
                        );
                        Ok((change, effect))
                    }
                    None => Ok((vec![], vec![])),
                }
            }
            Intent::BranchName(v) => {
                let change = vec![Change::BranchName(v.clone())];
                let effect = build_check_effect(local_model, global_model, process_id, None, |a| {
                    a.branch_name = v;
                });
                Ok((change, effect))
            }
            Intent::Currency(v) => {
                let change = vec![Change::Currency(v.clone())];
                let effect = build_check_effect(local_model, global_model, process_id, None, |a| {
                    a.currency = v;
                });
                Ok((change, effect))
            }
            Intent::Latitude(v) => {
                let mut loc = local_model.location().read();
                loc.latitude = v;
                let loc_for_effect = loc.clone();
                let change = vec![Change::Location(loc)];
                let effect = build_check_effect(local_model, global_model, process_id, None, |a| {
                    a.location = loc_for_effect;
                });
                Ok((change, effect))
            }
            Intent::Longitude(v) => {
                let mut loc = local_model.location().read();
                loc.longitude = v;
                let loc_for_effect = loc.clone();
                let change = vec![Change::Location(loc)];
                let effect = build_check_effect(local_model, global_model, process_id, None, |a| {
                    a.location = loc_for_effect;
                });
                Ok((change, effect))
            }
            Intent::Subscribe => {
                let effect = vec![Effect::Subscribe { process_id }];
                Ok((vec![], effect))
            }
            Intent::UnSubscribe => {
                let effect = vec![Effect::UnSubscribe { process_id }];
                Ok((vec![], effect))
            }
        },
        Message::Observe(observe) => match observe {
            Observe::Timeout => {
                let change = match local_model.async_state().read() {
                    AsyncState::Loading { .. } => vec![Change::ShowDialog(Dialog::Show)],
                    _ => vec![],
                };
                Ok((change, vec![]))
            }
            Observe::Result(result) => {
                let change = match result {
                    AsyncState::Success { .. } => {
                        vec![
                            Change::ShowDialog(Default::default()),
                            Change::CompanyName(Default::default()),
                            Change::BranchName(Default::default()),
                            Change::Currency(Default::default()),
                            Change::Location(Default::default()),
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
            Observe::Refresh => {
                let effect =
                    build_check_effect(local_model, global_model, process_id, None, |_| {});
                Ok((vec![], effect))
            }
        },
    }
}

pub fn update(msg: Change, local_model: &impl LocalModel, _global_model: &impl GlobalModel) {
    match msg {
        Change::ShowDialog(i) => local_model.show_dialog().set(i),
        Change::CompanyName(i) => local_model.company_name().set(i),
        Change::BranchName(i) => local_model.branch_name().set(i),
        Change::Currency(i) => local_model.currency().set(i),
        Change::Location(i) => local_model.location().set(i),
        Change::AsyncState(i) => local_model.async_state().set(i),
    }
}

pub async fn effect(msg: Effect, context: UiContext) -> Result<()> {
    match msg {
        Effect::SpawnTimer { process_id } => {
            Rt::spawn_local(async move {
                Rt::sleep(Duration::from_secs(5)).await;
                let _ = context
                    .sender_to_commander
                    .async_send(process_id, Message::Observe(Observe::Timeout))
                    .await;
            });
        }
        Effect::Submit {
            process_id,
            async_input,
            is_to_server,
        } => {
            handle_submit(process_id, async_input, context, is_to_server).await?;
        }
        Effect::Check {
            process_id,
            async_input,
        } => {
            handle_check(process_id, async_input, context).await?;
        }
        Effect::Subscribe { process_id } => {
            handle_subscribe(process_id, context).await?;
        }
        Effect::UnSubscribe { process_id } => {
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
    let new_uuid = BranchUuid::from(UuidType::from(Id::generate()));

    let data: TypeOperationClientInput = Arc::new(Input {
        user_uuid: input.user_uuid.clone(),
        new_uuid,
        company_belong: input.company_belong.clone(),
        branch_name: input.branch_name.clone(),
        currency: input.currency.clone(),
        location: input.location.clone(),
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
    let new_uuid = BranchUuid::from(UuidType::from(Id::generate()));

    let data: TypeOperationClientInput = Arc::new(Input {
        user_uuid: input.user_uuid.clone(),
        new_uuid,
        company_belong: input.company_belong.clone(),
        branch_name: input.branch_name.clone(),
        currency: input.currency.clone(),
        location: input.location.clone(),
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
