use crate::domain::Error;
use crate::domain::Input;
use crate::domain::MyResult;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use crate::domain::UserIdError;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use infrastructure::actors::MpscSender;
use infrastructure::actors::Sender;
use infrastructure::jwt::JsonWebTokenType;
use kernel::client::Cache;
use kernel::client::DialogSignalAdapter;
use kernel::new_types::UserUuid;
use kernel::types::DatabaseRead;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use utility::cache::CacheStruct;
use utility::cache::ResourceName;
use utility::cache::TraitOperationClientError;
use utility::cache::TraitOperationClientInput;
use utility::cache::TraitOperationClientOk;
use utility::cache::TypeOperationClientInput;
use utility::cache::TypeOperationClientResult;
use utility::cache::new_resource_name;
use utility::process_manager::MessageToProcessManager;
use utility::process_manager::ProcessId;
use utility::process_manager::UserConsent;
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

#[derive(Debug, Clone)]
pub enum Message {
    Submit,
    Consent(UserConsent),
    UserId(String),
    Password(String),
}

impl MessageTrait for Message {}

type Type1 = Input;
type Type2 = Input;
type Type3 = MyResult;
type Type4 = MyResult;

pub trait GlobalModel: Model + 'static {
    fn is_auth_loading(&self) -> impl HashimSignal<bool>;
    fn user_uuid(&self) -> impl HashimSignal<Option<UserUuid>>;
    fn user_name(&self) -> impl HashimSignal<Option<String>>;
    fn user_id(&self) -> impl HashimSignal<String>;
    fn password(&self) -> impl HashimSignal<String>;
}

pub trait LocalModel: 'static {
    fn process_id(&self) -> impl HashimSignal<Option<ProcessId>>;
    fn show_dialog(&self) -> impl HashimSignal<Dialog>;
    fn error_user_id(&self) -> impl HashimSignal<Option<String>>;
    fn error_password(&self) -> impl HashimSignal<Option<String>>;
}

fn apply_on_the_model(
    output: &Type4,
    local_model: Arc<impl LocalModel>,
    global_model: Arc<impl GlobalModel>,
) {
    match output {
        Ok(ok) => {
            local_model.error_user_id().reset();
            local_model.error_password().reset();

            global_model.user_uuid().set(Some(ok.user_uuid.clone()));
            global_model.user_name().set(ok.user_name.clone());
            global_model.user_id().set(ok.user_id.clone());
        }
        Err(business_error) => {
            local_model.error_user_id().set(
                business_error
                    .user_id
                    .as_ref()
                    .map(|_| String::from("user not exist")),
            );
            local_model.error_password().set(
                business_error
                    .password
                    .as_ref()
                    .map(|_| String::from("wrong password")),
            );
        }
    }
}

pub async fn update_generic(
    message: Message,
    local_model: Arc<impl LocalModel>,
    mut context: UiContext<impl GlobalModel>,
) -> Result<()> {
    match message {
        Message::Submit => {
            handle_submit(
                context.sender_to_error,
                context.model,
                local_model,
                context.cache,
                context.sender_to_process_manager,
            )
            .await?;
        }
        Message::Consent(i) => {
            context
                .sender_to_process_manager
                .send(MessageToProcessManager::FromUser {
                    process_id: local_model
                        .process_id()
                        .read()
                        .context("process id not found")?,
                    consent: i,
                })
                .await?;
        }
        Message::UserId(i) => {
            context.model.user_id().set(i);
        }
        Message::Password(i) => {
            context.model.password().set(i);
        }
    }

    Ok(())
}

fn build_input(global_model: Arc<impl GlobalModel>) -> Result<Type1> {
    Ok(Input {
        user_id: global_model.user_id().read(),
        password: global_model.password().read(),
    })
}

async fn handle_submit(
    sender_to_error: MpscSender<anyhow::Error>,
    global_model: Arc<impl GlobalModel>,
    local_model: Arc<impl LocalModel>,
    cache: CacheStruct,
    sender_to_process_manager: MpscSender<MessageToProcessManager>,
) -> Result<()> {
    if global_model.is_auth_loading().read() {
        return Ok(());
    }
    global_model.is_auth_loading().set(true);

    let process_id = ProcessId::default();
    local_model.process_id().set(Some(process_id));

    let dialog_signal_adapter = Arc::new(DialogSignalAdapter(local_model.show_dialog()));

    let data = build_input(global_model.clone())?;
    let data: TypeOperationClientInput = Arc::new(data);

    let global_model1 = global_model.clone();
    handle_fall_back(
        sender_to_error,
        cache,
        sender_to_process_manager,
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

            apply_on_the_model(&result, local_model.clone(), global_model.clone());

            Ok(result.is_ok())
        },
    )
    .await?;

    global_model1.is_auth_loading().reset();

    Ok(())
}
