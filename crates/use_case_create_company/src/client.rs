use crate::domain::CompanyNameError;
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
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::new_types::UuidType;
use kernel::types::Currency;
use kernel::types::DatabaseRead;
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
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UiContext;
use utility::ui_orchestration::GenricAsyncState;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[new_resource_name("companies")];

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

pub trait GlobalModel {
    fn user_uuid(&self) -> Option<UserUuid>;
}

pub(crate) type AsyncState = GenricAsyncState<AsyncInput, Ok, Error>;

pub trait LocalModel {
    fn show_dialog(&self) -> impl HashimSignal<Dialog>;
    fn company_name(&self) -> impl HashimSignal<String>;
    fn currency(&self) -> impl HashimSignal<Currency>;
    fn async_state(&self) -> impl HashimSignal<AsyncState>;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AsyncInput {
    pub user_uuid: UserUuid,
    pub company_name: String,
    pub currency: Currency,
}

pub fn error_company_name(local_model: &impl LocalModel) -> Option<CompanyNameError> {
    match local_model.async_state().read() {
        AsyncState::Failure { error, .. } => error.company_name,
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum Change {
    ShowDialog(Dialog),
    CompanyName(String),
    Currency(Currency),
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
}

#[derive(Debug, Clone)]
pub enum Intent {
    Clean,
    CompanyName(String),
    Consent(UserConsent),
    Currency(Currency),
    Submit,
}

#[derive(Debug, Clone)]
pub enum Observe {
    Timeout,
    Result(AsyncState),
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
    mutate: impl FnOnce(&mut AsyncInput),
) -> Vec<Effect> {
    let Some(user_uuid) = global_model.user_uuid() else {
        return vec![];
    };
    let mut a = AsyncInput {
        user_uuid,
        company_name: local_model.company_name().read(),
        currency: local_model.currency().read(),
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
        if let Intent::Consent(_) = a {
        } else {
            if local_model.async_state().read().is_loading() {
                return (vec![], vec![]);
            }
        }
    }

    match msg {
        Message::Intent(intent) => match intent {
            Intent::Clean => {
                let change = vec![
                    Change::ShowDialog(Default::default()),
                    Change::CompanyName(Default::default()),
                    Change::Currency(Default::default()),
                    Change::AsyncState(AsyncState::Idle),
                ];
                (change, vec![])
            }
            Intent::CompanyName(v) => {
                let change = vec![Change::CompanyName(v.clone())];
                let effect = build_check_effect(local_model, global_model, |a| {
                    a.company_name = v;
                });
                (change, effect)
            }
            Intent::Currency(v) => {
                let change = vec![Change::Currency(v.clone())];
                let effect = build_check_effect(local_model, global_model, |a| {
                    a.currency = v;
                });
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
                        let async_input = AsyncInput {
                            user_uuid,
                            company_name: local_model.company_name().read(),
                            currency: local_model.currency().read(),
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
            Intent::Submit => {
                let Some(user_uuid) = global_model.user_uuid() else {
                    return (vec![], vec![]);
                };

                let async_input = AsyncInput {
                    user_uuid,
                    company_name: local_model.company_name().read(),
                    currency: local_model.currency().read(),
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
                            Change::CompanyName(Default::default()),
                            Change::Currency(Default::default()),
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
        },
    }
}

pub fn update(msg: Change, local_model: &impl LocalModel, _global_model: &impl GlobalModel) {
    match msg {
        Change::ShowDialog(i) => local_model.show_dialog().set(i),
        Change::CompanyName(i) => local_model.company_name().set(i),
        Change::Currency(i) => local_model.currency().set(i),
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
    }
    Ok(())
}

async fn handle_submit(
    process_id: ProcessId,
    input: AsyncInput,
    mut context: UiContext,
    is_to_server: bool,
) -> Result<()> {
    let new_uuid = CompanyUuid::from(UuidType::from(Id::generate()));

    let data: TypeOperationClientInput = Arc::new(Input {
        user_uuid: input.user_uuid.clone(),
        new_uuid,
        company_name: input.company_name.clone(),
        currency: input.currency.clone(),
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
    let new_uuid = CompanyUuid::from(UuidType::from(Id::generate()));

    let data: TypeOperationClientInput = Arc::new(Input {
        user_uuid: input.user_uuid.clone(),
        new_uuid,
        company_name: input.company_name.clone(),
        currency: input.currency.clone(),
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
