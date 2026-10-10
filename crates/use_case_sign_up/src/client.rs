use crate::domain::Error;
use crate::domain::Input;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use crate::domain::UserIdError;
use crate::domain::UserNameError;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use infrastructure::jwt::JsonWebTokenType;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use kernel::client::Cache;
use kernel::new_types::UserUuid;
use kernel::new_types::UuidType;
use kernel::types::DatabaseRead;
use kernel::types::MyErrorTrait;
use serde::Deserialize;
use serde::Serialize;
use std::any::Any;
use std::fmt::Debug;
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
use utility::types::MakeOptionIfEmpty;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UiContext;
use utility::ui_orchestration::GenricAsyncState;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;

const RESOURCES_NAME_TO_POKE: &[ResourceName] = &[new_resource_name("users")];

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

    let state_less_operation = Ok {
        new_uuid: input.user_uuid.clone(),
        user_id: input.user_id.clone(),
        user_name: input.name.clone(),
        hashed_password: String::new(),
        jwt: JsonWebTokenType(String::new()),
    };

    Ok(Ok(Arc::new(state_less_operation)))
}

pub(crate) type AsyncState = GenricAsyncState<AsyncInput, Ok, Error>;

pub trait GlobalModel {
    fn user_name(&self) -> impl HashimSignal<Option<String>>;
    fn user_id(&self) -> impl HashimSignal<String>;
    fn password(&self) -> impl HashimSignal<String>;
}

pub trait LocalModel {
    fn show_dialog(&self) -> impl HashimSignal<Dialog>;
    fn async_state(&self) -> impl HashimSignal<AsyncState>;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AsyncInput {
    pub name: Option<String>,
    pub user_id: String,
    pub password: String,
}

pub fn user_uuid(local_model: &impl LocalModel) -> Option<UserUuid> {
    match local_model.async_state().read() {
        AsyncState::Success { ok, .. } => Some(ok.new_uuid),
        _ => None,
    }
}

pub fn user_name(local_model: &impl LocalModel) -> Option<String> {
    match local_model.async_state().read() {
        AsyncState::Success { ok, .. } => ok.user_name,
        _ => None,
    }
}

pub fn error_user_id(local_model: &impl LocalModel) -> Option<UserIdError> {
    match local_model.async_state().read() {
        AsyncState::Failure { error, .. } => error.user_id,
        _ => None,
    }
}

pub fn error_user_name(local_model: &impl LocalModel) -> Option<UserNameError> {
    match local_model.async_state().read() {
        AsyncState::Failure { error, .. } => error.name,
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum Change {
    ShowDialog(Dialog),
    UserName(Option<String>),
    UserId(String),
    Password(String),
    AsyncState(AsyncState),
}

#[derive(Debug, Clone)]
pub enum Effect {
    SpawnTimer,
    Submit {
        async_input: AsyncInput,
        is_to_server: bool,
    },
}

#[derive(Debug, Clone)]
pub enum Intent {
    Submit,
    Consent(UserConsent),
    UserName(String),
    UserId(String),
    Password(String),
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

pub fn reduce(
    msg: Message,
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Result<(Vec<Change>, Vec<Effect>)> {
    if let Message::Intent(ref a) = msg {
        if let Intent::Consent(_) = a {
        } else {
            if local_model.async_state().read().is_loading() {
                return Ok((vec![], vec![]));
            }
        }
    }

    match msg {
        Message::Intent(intent) => match intent {
            Intent::Submit => {
                let async_input = AsyncInput {
                    name: global_model.user_name().read(),
                    user_id: global_model.user_id().read(),
                    password: global_model.password().read(),
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
                        vec![Effect::SpawnTimer],
                    ),
                    UserConsent::DontWaitForServerResponse => {
                        let async_input = AsyncInput {
                            name: global_model.user_name().read(),
                            user_id: global_model.user_id().read(),
                            password: global_model.password().read(),
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

                Ok((change, effect))
            }
            Intent::UserName(v) => {
                let change = vec![Change::UserName(v.none_if_empty())];
                Ok((change, vec![]))
            }
            Intent::UserId(v) => {
                let change = vec![Change::UserId(v)];
                Ok((change, vec![]))
            }
            Intent::Password(v) => {
                let change = vec![Change::Password(v)];
                Ok((change, vec![]))
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
                let change = vec![Change::AsyncState(result)];
                Ok((change, vec![]))
            }
        },
    }
}

pub fn update(msg: Change, local_model: &impl LocalModel, global_model: &impl GlobalModel) {
    match msg {
        Change::ShowDialog(i) => local_model.show_dialog().set(i),
        Change::UserName(i) => global_model.user_name().set(i),
        Change::UserId(i) => global_model.user_id().set(i),
        Change::Password(i) => global_model.password().set(i),
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
    }
    Ok(())
}

async fn handle_submit(
    process_id: ProcessId,
    input: AsyncInput,
    mut context: UiContext,
    is_to_server: bool,
) -> Result<()> {
    let data: TypeOperationClientInput = Arc::new(Input {
        user_uuid: UserUuid::from(UuidType::from(Id::generate())),
        name: input.name.clone(),
        user_id: input.user_id.clone(),
        password: input.password.clone(),
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
