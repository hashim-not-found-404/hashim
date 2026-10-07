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
use infrastructure::actors::Sender;
use infrastructure::jwt::JsonWebTokenType;
use kernel::client::Cache;
use kernel::new_types::UserUuid;
use kernel::types::DatabaseRead;
use serde::Deserialize;
use serde::Serialize;
use std::any::Any;
use std::fmt::Debug;
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
use utility::ui_effect::Commander;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UiContext;
use utility::ui_orchestration::handle_fall_back;
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

pub trait GlobalModel {
    fn user_id(&self) -> impl HashimSignal<String>;
    fn password(&self) -> impl HashimSignal<String>;
}

pub trait LocalModel {
    fn show_dialog(&self) -> impl HashimSignal<Dialog>;
    fn async_state(&self) -> impl HashimSignal<AsyncState>;
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
    user_id: String,
    password: String,
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
    UserId(String),
    Password(String),
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

pub fn is_auth_loading(local_model: &impl LocalModel) -> bool {
    let a = local_model.async_state().read();
    match a {
        AsyncState::Loading { .. } => true,
        _ => false,
    }
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
    let a = local_model.async_state().read();
    match a {
        AsyncState::Idle => None,
        AsyncState::Loading { .. } => None,
        AsyncState::Success { .. } => None,
        AsyncState::Failure { error, .. } => error.user_id,
    }
}

pub fn error_password(local_model: &impl LocalModel) -> Option<PasswordError> {
    let a = local_model.async_state().read();
    match a {
        AsyncState::Idle => None,
        AsyncState::Loading { .. } => None,
        AsyncState::Success { .. } => None,
        AsyncState::Failure { error, .. } => error.password,
    }
}

pub fn reduce(
    msg: Message,
    process_id: ProcessId,
    local_model: &impl LocalModel,
    global_model: &impl GlobalModel,
) -> Result<(Vec<Change>, Vec<Effect>)> {
    match msg {
        Message::Intent(intent) => match intent {
            Intent::Submit => {
                if is_auth_loading(local_model) {
                    return Ok((vec![], vec![]));
                }

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
            Intent::UserId(v) => {
                let change = vec![Change::UserId(v.clone())];
                let effect = vec![];
                Ok((change, effect))
            }
            Intent::Password(v) => {
                let change = vec![Change::Password(v.clone())];
                let effect = vec![];
                Ok((change, effect))
            }
        },
        Message::Observe(observe) => match observe {
            Observe::ShowDialog => Ok((vec![Change::ShowDialog(Dialog::Show)], vec![])),
            Observe::HideDialog => Ok((vec![Change::ShowDialog(Dialog::Hide)], vec![])),
            Observe::Result(result) => {
                let change = vec![Change::AsyncState(result)];
                Ok((change, vec![]))
            }
        },
    }
}

pub fn update(msg: Change, local_model: &impl LocalModel, global_model: &impl GlobalModel) {
    match msg {
        Change::Password(i) => global_model.password().set(i),
        Change::ShowDialog(i) => local_model.show_dialog().set(i),
        Change::UserId(i) => global_model.user_id().set(i),
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

    let data: TypeOperationClientInput = Arc::new(Input {
        user_id: input.user_id.clone(),
        password: input.password.clone(),
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
