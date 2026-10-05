use crate::domain::Error;
use crate::domain::Input;
use crate::domain::MyResult;
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
use utility::ui_effect::Model;
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

pub trait GlobalModel: Model + 'static {
    fn is_auth_loading(&self) -> impl HashimSignal<bool>;
    fn user_uuid(&self) -> impl HashimSignal<Option<UserUuid>>;
    fn user_name(&self) -> impl HashimSignal<Option<String>>;
    fn user_id(&self) -> impl HashimSignal<String>;
    fn password(&self) -> impl HashimSignal<String>;
}

pub trait LocalModel: 'static {
    fn show_dialog(&self) -> impl HashimSignal<Dialog>;
    fn error_user_id(&self) -> impl HashimSignal<Option<UserIdError>>;
    fn error_password(&self) -> impl HashimSignal<Option<PasswordError>>;
}

#[derive(Debug, Clone)]
pub enum Change {
    ErrorPassword(Option<PasswordError>),
    ErrorUserId(Option<UserIdError>),
    IsAuthLoading(bool),
    Password(String),
    ShowDialog(Dialog),
    UserId(String),
    UserName(Option<String>),
    UserUuid(Option<UserUuid>),
}

#[derive(Debug, Clone)]
pub enum Effect {
    Submit {
        process_id: ProcessId,
        user_id: String,
        password: String,
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
    Result(MyResult),
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
                if global_model.is_auth_loading().read() {
                    return Ok((vec![], vec![]));
                }
                let change = vec![
                    Change::IsAuthLoading(true),
                    Change::ShowDialog(Default::default()),
                    Change::ErrorUserId(Default::default()),
                    Change::ErrorPassword(Default::default()),
                ];
                let effect = vec![Effect::Submit {
                    process_id,
                    user_id: global_model.user_id().read(),
                    password: global_model.password().read(),
                }];
                Ok((change, effect))
            }
            Intent::Consent(v) => {
                let change = vec![Change::ShowDialog(Dialog::Hide)];
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
                let change = match result {
                    Ok(ok) => vec![
                        Change::IsAuthLoading(false),
                        Change::UserUuid(Some(ok.user_uuid.clone())),
                        Change::UserName(ok.user_name.clone()),
                        Change::ErrorUserId(None),
                        Change::ErrorPassword(None),
                    ],
                    Err(a) => vec![
                        Change::IsAuthLoading(false),
                        Change::ErrorUserId(a.user_id),
                        Change::ErrorPassword(a.password),
                    ],
                };
                Ok((change, vec![]))
            }
        },
    }
}

pub fn update(msg: Change, local_model: &impl LocalModel, global_model: &impl GlobalModel) {
    match msg {
        Change::Password(i) => global_model.password().set(i),
        Change::ShowDialog(i) => local_model.show_dialog().set(i),
        Change::IsAuthLoading(i) => global_model.is_auth_loading().set(i),
        Change::UserUuid(i) => global_model.user_uuid().set(i),
        Change::UserName(i) => global_model.user_name().set(i),
        Change::ErrorUserId(i) => local_model.error_user_id().set(i),
        Change::ErrorPassword(i) => local_model.error_password().set(i),
        Change::UserId(i) => global_model.user_id().set(i),
    }
}

pub async fn effect(msg: Effect, mut context: UiContext) -> Result<()> {
    match msg {
        Effect::Submit {
            process_id,
            user_id,
            password,
        } => {
            handle_submit(process_id, &Input { user_id, password }, context).await?;
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
                .send(process_id, Message::Observe(Observe::Result(result)));

            Ok(is_ok)
        },
    )
    .await?;

    Ok(())
}
