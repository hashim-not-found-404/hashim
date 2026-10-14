use crate::domain::Error;
use crate::domain::Input;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use crate::domain::UserIdError;
use crate::domain::UserNameError;
use anyhow::Result;
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
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use utility::cache::ResourceName;
use utility::cache::TraitOperationClientError;
use utility::cache::TraitOperationClientInput;
use utility::cache::TraitOperationClientOk;
use utility::cache::TypeOperationClientError;
use utility::cache::TypeOperationClientResult;
use utility::cache::new_resource_name;
use utility::types::MakeOptionIfEmpty;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::Replier;
use utility::ui_effect::UiContext;
use utility::ui_orchestration::GenricAsyncState;
use utility::ui_orchestration::UseCaseClient;
use utility::ui_orchestration::UserConsent;
use utility::ui_orchestration::handle_submit;
use utility_ui::domain::Dialog;

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
    fn user_name(&self) -> Option<String>;
    fn user_id(&self) -> String;
    fn password(&self) -> String;

    fn set_user_name(&mut self, v: Option<String>);
    fn set_user_id(&mut self, v: String);
    fn set_password(&mut self, v: String);
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct LocalModel {
    pub show_dialog: Dialog,
    pub async_state: AsyncState,
}

impl LocalModel {
    pub fn user_uuid(&self) -> Option<UserUuid> {
        match self.async_state.clone() {
            AsyncState::Success { ok, .. } => Some(ok.new_uuid),
            _ => None,
        }
    }

    pub fn user_name(&self) -> Option<String> {
        match self.async_state.clone() {
            AsyncState::Success { ok, .. } => ok.user_name,
            _ => None,
        }
    }

    pub fn error_user_id(&self) -> Option<UserIdError> {
        match self.async_state.clone() {
            AsyncState::Failure { error, .. } => error.user_id,
            _ => None,
        }
    }

    pub fn error_user_name(&self) -> Option<UserNameError> {
        match self.async_state.clone() {
            AsyncState::Failure { error, .. } => error.name,
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AsyncInput {
    pub name: Option<String>,
    pub user_id: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub enum LocalChange {
    ShowDialog(Dialog),
    AsyncState(AsyncState),
}

#[derive(Debug, Clone)]
pub enum GlobalChange {
    UserName(Option<String>),
    UserId(String),
    Password(String),
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

pub fn update(
    msg: Message,
    local_model: &LocalModel,
    global_model: &impl GlobalModel,
) -> (Vec<LocalChange>, Vec<GlobalChange>, Vec<Effect>) {
    if let Message::Intent(ref a) = msg {
        if let Intent::Consent(_) = a {
        } else {
            if local_model.async_state.clone().is_loading() {
                return (vec![], vec![], vec![]);
            }
        }
    }

    match msg {
        Message::Intent(intent) => match intent {
            Intent::Submit => {
                let async_input = AsyncInput {
                    name: global_model.user_name(),
                    user_id: global_model.user_id(),
                    password: global_model.password(),
                };

                let local_change = vec![
                    LocalChange::AsyncState(AsyncState::Loading {
                        input: async_input.clone(),
                    }),
                    LocalChange::ShowDialog(Dialog::Hide),
                ];

                let effect = vec![
                    Effect::SpawnTimer,
                    Effect::Submit {
                        async_input,
                        is_to_server: true,
                    },
                ];

                (local_change, vec![], effect)
            }
            Intent::Consent(v) => {
                let (local_change, effect) = match v {
                    UserConsent::CancelOperation => (
                        vec![
                            LocalChange::ShowDialog(Dialog::Hide),
                            LocalChange::AsyncState(AsyncState::Idle),
                        ],
                        vec![],
                    ),
                    UserConsent::WaitForServerResponse => (
                        vec![LocalChange::ShowDialog(Dialog::Hide)],
                        vec![Effect::SpawnTimer],
                    ),
                    UserConsent::DontWaitForServerResponse => {
                        let async_input = AsyncInput {
                            name: global_model.user_name(),
                            user_id: global_model.user_id(),
                            password: global_model.password(),
                        };

                        (
                            vec![LocalChange::ShowDialog(Dialog::Hide)],
                            vec![Effect::Submit {
                                async_input,
                                is_to_server: false,
                            }],
                        )
                    }
                };

                (local_change, vec![], effect)
            }
            Intent::UserName(v) => {
                let global_change = vec![GlobalChange::UserName(v.none_if_empty())];
                (vec![], global_change, vec![])
            }
            Intent::UserId(v) => {
                let global_change = vec![GlobalChange::UserId(v)];
                (vec![], global_change, vec![])
            }
            Intent::Password(v) => {
                let global_change = vec![GlobalChange::Password(v)];
                (vec![], global_change, vec![])
            }
        },
        Message::Observe(observe) => match observe {
            Observe::Timeout => {
                let local_change = match local_model.async_state.clone() {
                    AsyncState::Loading { .. } => vec![LocalChange::ShowDialog(Dialog::Show)],
                    _ => vec![],
                };
                (local_change, vec![], vec![])
            }
            Observe::Result(result) => {
                let local_change = vec![LocalChange::AsyncState(result)];
                (local_change, vec![], vec![])
            }
        },
    }
}

pub fn local_apply(msg: LocalChange, local_model: &mut LocalModel) {
    match msg {
        LocalChange::ShowDialog(i) => local_model.show_dialog = i,
        LocalChange::AsyncState(i) => local_model.async_state = i,
    }
}

pub fn global_apply(msg: GlobalChange, global_model: &mut impl GlobalModel) {
    match msg {
        GlobalChange::Password(i) => global_model.set_password(i),
        GlobalChange::UserId(i) => global_model.set_user_id(i),
        GlobalChange::UserName(i) => global_model.set_user_name(i),
    }
}

pub async fn effect(msg: Effect, replier: Replier, context: UiContext) -> Result<()> {
    match msg {
        Effect::SpawnTimer => {
            Rt::spawn_local(async move {
                Rt::sleep(Duration::from_secs(5)).await;
                let _ = replier(Box::new(Message::Observe(Observe::Timeout))).await;
            });
        }
        Effect::Submit {
            async_input,
            is_to_server,
        } => {
            handle_submit::<Wire>(replier, async_input, context, is_to_server).await?;
        }
    }
    Ok(())
}

pub struct Wire;

impl UseCaseClient for Wire {
    type Input = Input;
    type AsyncInput = AsyncInput;
    type Ok = Ok;
    type Error = Error;
    type Message = Message;

    fn msg_success_submit(input: Self::AsyncInput, ok: Self::Ok) -> Self::Message {
        Message::Observe(Observe::Result(AsyncState::Success { input, ok }))
    }

    fn msg_failure_submit(input: Self::AsyncInput, error: Self::Error) -> Self::Message {
        Message::Observe(Observe::Result(AsyncState::Failure { input, error }))
    }

    fn msg_timeout() -> Self::Message {
        Message::Observe(Observe::Timeout)
    }

    fn build_input(input: &Self::AsyncInput) -> Self::Input {
        Input {
            user_uuid: UserUuid::from(UuidType::from(Id::generate())),
            name: input.name.clone(),
            user_id: input.user_id.clone(),
            password: input.password.clone(),
        }
    }
}
