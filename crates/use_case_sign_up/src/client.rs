use crate::domain::Error;
use crate::domain::Input;
use crate::domain::MyResult;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use infrastructure::actors::MpscSender;
use infrastructure::actors::Receiver;
use infrastructure::actors::Sender;
use infrastructure::jwt::JsonWebTokenType;
use kernel::client::Cache;
use kernel::client::DialogSignalAdapter;
use kernel::new_types::UserUuid;
use kernel::types::DatabaseRead;
use kernel::types::MyErrorTrait;
use std::any::Any;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use utility::cache::CacheStruct;
use utility::cache::CachingStrategy;
use utility::cache::Response;
use utility::cache::Subscribe;
use utility::cache::TraitOperationClientError;
use utility::cache::TraitOperationClientInput;
use utility::cache::TraitOperationClientOk;
use utility::cache::TypeOperationClientInput;
use utility::cache::TypeOperationClientResult;
use utility::dtos::TxnNumber;
use utility::process_manager::MessageToProcessManager;
use utility::process_manager::ProcessId;
use utility::process_manager::UserConsent;
use utility::types::MakeOptionIfEmpty;
use utility::ui_effect::Aborters;
use utility::ui_effect::MessageTrait;
use utility::ui_orchestration::handle_fall_back;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;

impl TraitOperationClientOk for Ok {
    fn subs_to_poke(&self) -> &'static [Subscribe] {
        todo!()
    }
}

impl TraitOperationClientError for Error {
    fn subs_to_poke(&self) -> &'static [Subscribe] {
        todo!()
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

#[derive(Debug, Clone)]
pub enum Message {
    Submit,
    Consent(UserConsent),
    UserName(String),
    UserId(String),
    Password(String),
    GoToSignIn,
}

impl MessageTrait for Message {}

type Type1 = Input;
type Type2 = Input;
type Type3 = MyResult;
type Type4 = MyResult;

pub trait GlobalModel: 'static {
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
    fn error_user_name(&self) -> impl HashimSignal<Option<String>>;
}

fn apply_on_the_model(output: &Type4, local_model: Arc<impl LocalModel>) {
    match output {
        Ok(_) => {
            local_model.error_user_id().reset();
            local_model.error_user_name().reset();
        }
        Err(business_error) => {
            local_model.error_user_id().set(
                business_error
                    .user_id
                    .as_ref()
                    .map(|_| String::from("duplicated user")),
            );
            local_model
                .error_user_name()
                .set(business_error.name.clone());
        }
    }
}

pub async fn update_generic(
    message: Message,
    global_model: Arc<impl GlobalModel>,
    local_model: Arc<impl LocalModel>,
    cache: CacheStruct,
    mut sender_to_process_manager: MpscSender<MessageToProcessManager>,
    aborters: Aborters,
) -> Result<()> {
    match message {
        Message::Submit => {
            handle_submit(global_model, local_model, cache, sender_to_process_manager).await?;
        }
        Message::Consent(i) => {
            sender_to_process_manager
                .send(MessageToProcessManager::FromUser {
                    process_id: local_model
                        .process_id()
                        .read()
                        .context("process id not found")?,
                    consent: i,
                })
                .await?;
        }
        Message::UserName(i) => {
            global_model.user_name().set(i.none_if_empty());
            handle_check(global_model, local_model, cache).await?;
        }
        Message::UserId(i) => {
            global_model.user_id().set(i);
            handle_check(global_model, local_model, cache).await?;
        }
        Message::Password(i) => {
            global_model.password().set(i);
            handle_check(global_model, local_model, cache).await?;
        }
        Message::GoToSignIn => todo!(),
    }

    Ok(())
}

fn build_input(
    global_model: Arc<impl GlobalModel>,
    local_model: Arc<impl LocalModel>,
) -> Result<Type1> {
    Ok(Input {
        user_uuid: global_model
            .user_uuid()
            .read()
            .context("user uuid not found")?,
        name: global_model.user_name().read(),
        user_id: global_model.user_id().read(),
        password: global_model.password().read(),
    })
}

fn handle_clean(local_model: Arc<impl LocalModel>) {
    local_model.process_id().reset();
    local_model.error_user_id().reset();
    local_model.error_user_name().reset();
}

async fn handle_submit(
    global_model: Arc<impl GlobalModel>,
    local_model: Arc<impl LocalModel>,
    cache: CacheStruct,
    sender_to_process_manager: MpscSender<MessageToProcessManager>,
) -> Result<()> {
    let process_id = ProcessId::default();
    local_model.process_id().set(Some(process_id));

    let dialog_signal_adapter = Arc::new(DialogSignalAdapter(local_model.show_dialog()));

    let data = build_input(global_model.clone(), local_model.clone())?;

    let data: TypeOperationClientInput = Arc::new(data);

    handle_fall_back(
        cache,
        sender_to_process_manager,
        dialog_signal_adapter,
        process_id,
        data,
        move |data| {
            let result = match data {
                Ok(ok) => {
                    let a = ok;
                    let a: Arc<dyn Any> = a;
                    let a: &Ok = a.downcast_ref().context("downcast error")?;
                    let a: Ok = a.clone();
                    Ok(a)
                }
                Err(err) => {
                    let a = err;
                    let a: Box<dyn Any> = a;
                    let a: Box<Error> = a.downcast().map_err(|_| anyhow!(""))?;
                    let a: Error = a.as_ref().clone();
                    Err(a)
                }
            };
            apply_on_the_model(&result, local_model.clone());

            let is_ok = result.is_ok();
            if is_ok {
                handle_clean(local_model.clone());
            }

            Ok(is_ok)
        },
    )
    .await?;

    global_model.clone().is_auth_loading().reset();

    Ok(())
}

async fn handle_check(
    global_model: Arc<impl GlobalModel>,
    local_model: Arc<impl LocalModel>,
    mut cache: CacheStruct,
) -> Result<()> {
    let data = build_input(global_model, local_model.clone())?;

    let data: TypeOperationClientInput = Arc::new(data);

    let mut receiver_to_response = cache
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
                    let a = ok;
                    let a: Arc<dyn Any> = a;
                    let a: &Ok = a.downcast_ref().context("downcast error")?;
                    let a: Ok = a.clone();
                    Ok(a)
                }
                Err(err) => {
                    let a = err;
                    let a: Box<dyn Any> = a;
                    let a: Box<Error> = a.downcast().map_err(|_| anyhow!(""))?;
                    let a: Error = a.as_ref().clone();
                    Err(a)
                }
            };
            apply_on_the_model(&result, local_model);
        }
    }

    Ok(())
}
