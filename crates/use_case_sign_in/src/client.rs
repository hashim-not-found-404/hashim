use crate::domain::Error;
use crate::domain::Input;
use crate::domain::Ok;
use crate::domain::PasswordError;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use crate::domain::UserIdError;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use infrastructure::jwt::JsonWebTokenType;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use kernel::client::Cache;
use kernel::new_types::UserUuid;
use kernel::types::DatabaseRead;
use serde::Deserialize;
use serde::Serialize;
use std::any::Any;
use std::fmt::Debug;
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
        None
    }

    fn state_less_check(&self) -> Option<TypeOperationClientError> {
        None
    }
}

pub async fn check_input<
    Ch: Cache,
    DBReader: for<'a> DatabaseRead<Db<'a> = Ch, Input = ReadInput, Output = ReadOutput>,
>(
    input: &Input,
    cache: &mut Ch,
) -> Result<TypeOperationClientResult> {
    let read_output = DBReader::read(
        cache,
        &ReadInput {
            user_id: input.user_id.clone(),
        },
    )
    .await?;

    let Some((user_uuid, cached_jwt, user_name)) =
        read_output.user_rowid_and_password_or_jwt_and_name
    else {
        return Ok(Err(Box::new(Error {
            user_id: Some(UserIdError::NotExist),
            password: None,
        })));
    };

    if cached_jwt.is_empty() {
        return Ok(Err(Box::new(Error {
            user_id: Some(UserIdError::NotExist),
            password: None,
        })));
    }

    Ok(Ok(Arc::new(Ok {
        user_uuid,
        user_id: input.user_id.clone(),
        user_name,
        jwt: JsonWebTokenType(cached_jwt),
    })))
}

pub(crate) type AsyncState = GenricAsyncState<AsyncInput, Ok, Error>;

pub trait GlobalModel {
    fn user_id(&self) -> impl HashimSignal<String>;
    fn password(&self) -> impl HashimSignal<String>;
}

pub trait LocalModel {
    fn show_dialog(&self) -> impl HashimSignal<Dialog>;
    fn async_state(&self) -> impl HashimSignal<AsyncState>;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AsyncInput {
    pub user_id: String,
    pub password: String,
}

pub fn user_uuid(local_model: &impl LocalModel) -> Option<UserUuid> {
    match local_model.async_state().read() {
        AsyncState::Success { ok, .. } => Some(ok.user_uuid),
        _ => None,
    }
}

pub fn user_id(local_model: &impl LocalModel) -> Option<String> {
    match local_model.async_state().read() {
        AsyncState::Success { ok, .. } => Some(ok.user_id),
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

pub fn error_password(local_model: &impl LocalModel) -> Option<PasswordError> {
    match local_model.async_state().read() {
        AsyncState::Failure { error, .. } => error.password,
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum Change {
    Password(String),
    ShowDialog(Dialog),
    UserId(String),
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

pub fn update(
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
            Intent::Submit => {
                let async_input = AsyncInput {
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
                        let async_input = AsyncInput {
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

                (change, effect)
            }
            Intent::UserId(v) => {
                let change = vec![Change::UserId(v)];
                (change, vec![])
            }
            Intent::Password(v) => {
                let change = vec![Change::Password(v)];
                (change, vec![])
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
                let change = vec![Change::AsyncState(result)];
                (change, vec![])
            }
        },
    }
}

pub fn apply(msg: Change, local_model: &impl LocalModel, global_model: &impl GlobalModel) {
    match msg {
        Change::Password(i) => global_model.password().set(i),
        Change::ShowDialog(i) => local_model.show_dialog().set(i),
        Change::UserId(i) => global_model.user_id().set(i),
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
