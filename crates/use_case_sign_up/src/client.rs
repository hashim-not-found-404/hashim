use crate::domain::Error;
use crate::domain::Input;
use crate::domain::MyResult;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use crate::domain::UserIdError;
use crate::domain::UserNameError;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use infrastructure::actors::Receiver;
use infrastructure::actors::Sender;
use infrastructure::jwt::JsonWebTokenType;
use infrastructure::row_id::Id;
use infrastructure::row_id::RowId;
use kernel::client::Cache;
use kernel::new_types::UserUuid;
use kernel::new_types::UuidType;
use kernel::types::DatabaseRead;
use kernel::types::MyErrorTrait;
use std::any::Any;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
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
use utility::types::MakeOptionIfEmpty;
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

    let state_less_operation = Ok {
        new_uuid: input.user_uuid.clone(),
        user_id: input.user_id.clone(),
        user_name: input.name.clone(),
        hashed_password: String::new(),
        jwt: JsonWebTokenType(String::new()),
    };

    Ok(Ok(Arc::new(state_less_operation)))
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
    fn error_user_name(&self) -> impl HashimSignal<Option<UserNameError>>;
}

#[derive(Debug, Clone)]
pub enum Change {
    ErrorUserId(Option<UserIdError>),
    ErrorUserName(Option<UserNameError>),
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
        name: Option<String>,
        user_id: String,
        password: String,
    },
    Consent {
        process_id: ProcessId,
        user_consent: UserConsent,
    },
    Check {
        process_id: ProcessId,
        name: Option<String>,
        user_id: String,
        password: String,
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
            Intent::Submit => {
                if global_model.is_auth_loading().read() {
                    return Ok((vec![], vec![]));
                }

                let change = vec![
                    Change::IsAuthLoading(true),
                    Change::ShowDialog(Default::default()),
                    Change::ErrorUserId(Default::default()),
                    Change::ErrorUserName(Default::default()),
                ];
                let effect = vec![Effect::Submit {
                    process_id,
                    name: global_model.user_name().read(),
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
            Intent::UserName(v) => {
                let name = v.none_if_empty();
                let change = vec![Change::UserName(name.clone())];
                let effect = vec![Effect::Check {
                    process_id,
                    name,
                    user_id: global_model.user_id().read(),
                    password: global_model.password().read(),
                }];
                Ok((change, effect))
            }
            Intent::UserId(v) => {
                let change = vec![Change::UserId(v.clone())];
                let effect = vec![Effect::Check {
                    process_id,
                    name: global_model.user_name().read(),
                    user_id: v,
                    password: global_model.password().read(),
                }];
                Ok((change, effect))
            }
            Intent::Password(v) => {
                let change = vec![Change::Password(v.clone())];
                let effect = vec![Effect::Check {
                    process_id,
                    name: global_model.user_name().read(),
                    user_id: global_model.user_id().read(),
                    password: v,
                }];
                Ok((change, effect))
            }
        },
        Message::Observe(observe) => match observe {
            Observe::ShowDialog => Ok((vec![Change::ShowDialog(Dialog::Show)], vec![])),
            Observe::HideDialog => Ok((vec![Change::ShowDialog(Dialog::Hide)], vec![])),
            Observe::SubmitResult(result) => {
                let change = match result {
                    Ok(ok) => vec![
                        Change::IsAuthLoading(Default::default()),
                        Change::UserUuid(Some(ok.new_uuid.clone())),
                        Change::UserName(ok.user_name.clone()),
                        Change::ErrorUserId(None),
                        Change::ErrorUserName(None),
                    ],
                    Err(a) => vec![
                        Change::IsAuthLoading(Default::default()),
                        Change::ErrorUserId(a.user_id),
                        Change::ErrorUserName(a.name),
                    ],
                };
                Ok((change, vec![]))
            }
            Observe::CheckResult(result) => {
                let change = match result {
                    Ok(ok) => vec![
                        Change::UserName(ok.user_name.clone()),
                        Change::ErrorUserId(None),
                        Change::ErrorUserName(None),
                    ],
                    Err(a) => vec![
                        Change::ErrorUserId(a.user_id),
                        Change::ErrorUserName(a.name),
                    ],
                };
                Ok((change, vec![]))
            }
        },
    }
}

pub fn update(msg: Change, local_model: &impl LocalModel, global_model: &impl GlobalModel) {
    match msg {
        Change::ShowDialog(i) => local_model.show_dialog().set(i),
        Change::UserUuid(i) => global_model.user_uuid().set(i),
        Change::UserName(i) => global_model.user_name().set(i),
        Change::UserId(i) => global_model.user_id().set(i),
        Change::Password(i) => global_model.password().set(i),
        Change::ErrorUserId(i) => local_model.error_user_id().set(i),
        Change::ErrorUserName(i) => local_model.error_user_name().set(i),
        Change::IsAuthLoading(i) => global_model.is_auth_loading().set(i),
    }
}

pub async fn effect(msg: Effect, mut context: UiContext) -> Result<()> {
    match msg {
        Effect::Submit {
            process_id,
            name,
            user_id,
            password,
        } => {
            let input = Input {
                user_uuid: UserUuid::from(UuidType::from(Id::generate())),
                name,
                user_id,
                password,
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
            name,
            user_id,
            password,
        } => {
            let input = Input {
                user_uuid: UserUuid::from(UuidType::from(Id::generate())),
                name,
                user_id,
                password,
            };
            handle_check(process_id, input, context).await?;
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
