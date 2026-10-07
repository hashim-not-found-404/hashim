use crate::cache::CacheStruct;
use crate::cache::CachingStrategy;
use crate::cache::ResourceName;
use crate::cache::Response;
use crate::cache::TypeOperationClientInput;
use crate::cache::TypeOperationClientResult;
use crate::dtos::TxnNumber;
use crate::handle_errors::handle_error_one_time;
use crate::process_manager::DialogType;
use crate::process_manager::MessageFromProcess;
use crate::process_manager::MessageToProcess;
use crate::process_manager::MessageToProcessManager;
use crate::process_manager::ProcessId;
use crate::ui_effect::Aborter;
use crate::ui_effect::MessageTrait;
use crate::ui_effect::UiContext;
use anyhow::Error;
use anyhow::Result;
use infrastructure::actors::Mpsc;
use infrastructure::actors::MpscSender;
use infrastructure::actors::MultiProducerSingleConsumer;
use infrastructure::actors::Receiver;
use infrastructure::actors::Sender;
use infrastructure::random_number::RandomNumber;
use infrastructure::random_number::Rn;
use infrastructure::runtime::JoinHandle;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

pub async fn handle_fall_back(
    sender_to_error: MpscSender<Error>,
    mut cache: CacheStruct,
    mut sender_to_process_manager: MpscSender<MessageToProcessManager>,
    dialog: DialogType,
    process_id: ProcessId,
    data: TypeOperationClientInput,
    f: impl Fn(TypeOperationClientResult) -> Result<bool> + Clone + 'static,
) -> Result<()> {
    let txn_number = TxnNumber::default();

    let f1 = f.clone();
    let data1 = data.clone();
    let mut cache1 = cache.clone();
    let mut sender_to_process_manager1 = sender_to_process_manager.clone();

    let mut handle = Rt::abortable_spawn_local(async move {
        handle_error_one_time(sender_to_error, async move || {
            let mut receiver_to_response = cache1
                .send_to_cache_actor(CachingStrategy::WriteServerOnly, txn_number, data1)
                .await?;

            match receiver_to_response.recv().await? {
                Response::CloseTheChannel | Response::ServerCannotBeReached => {}
                Response::Data {
                    is_response_from_server,
                    data,
                } => {
                    sender_to_process_manager1
                        .send(MessageToProcessManager::FromProcess {
                            process_id,
                            message: MessageFromProcess::Response {
                                is_response_from_server,
                                is_response_ok: f1(data)?,
                            },
                        })
                        .await?;
                }
            }

            Ok(())
        })
        .await;
    });

    let (sender, mut receiver_to_process) = Mpsc::channel();
    sender_to_process_manager
        .send(MessageToProcessManager::FromProcess {
            process_id,
            message: MessageFromProcess::Subscribe { sender, dialog },
        })
        .await?;

    match receiver_to_process.recv().await? {
        MessageToProcess::CancelOperation => {}
        MessageToProcess::FallBackToCache => {
            let mut receiver_to_response = cache
                .send_to_cache_actor(CachingStrategy::WriteCacheOnly, txn_number, data)
                .await?;

            match receiver_to_response.recv().await? {
                Response::CloseTheChannel | Response::ServerCannotBeReached => {}
                Response::Data {
                    is_response_from_server: _,
                    data,
                } => {
                    f(data)?;
                }
            }
        }
    }
    handle.abort().await;
    Ok(())
}

pub async fn spawn_listener(
    mut context: UiContext,
    list_of_subscribtion: &'static [ResourceName],
    process_id: ProcessId,
    msg: impl MessageTrait + Clone,
) -> Result<()> {
    let component_id = Rn::generate() as u16;

    context
        .cache
        .send_subs_to_cache_actor(component_id, list_of_subscribtion, move || {
            context.sender_to_commander.send(process_id, msg.clone());
        })
        .await?;

    let aborter = Aborter::new(move || {
        Rt::spawn_local(async move {
            handle_error_one_time(context.sender_to_error, async move || {
                context
                    .cache
                    .send_unsubs_to_cache_actor(component_id)
                    .await?;
                Ok(())
            })
            .await;
        });
    });

    context.aborters.register(process_id, aborter);
    Ok(())
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(bound(
    deserialize = "Input: DeserializeOwned, Ok: DeserializeOwned, Error: DeserializeOwned",
    serialize = "Input: Serialize, Ok: Serialize, Error: Serialize",
))]
pub enum GenricAsyncState<Input, Ok, Error>
where
    Input: Debug + Clone,
    Ok: Debug + Clone,
    Error: Debug + Clone,
{
    #[default]
    Idle,
    Loading {
        input: Input,
    },
    Success {
        input: Input,
        ok: Ok,
    },
    Failure {
        input: Input,
        error: Error,
    },
}

impl<Input, Ok, Error> GenricAsyncState<Input, Ok, Error>
where
    Input: Debug + Clone,
    Ok: Debug + Clone,
    Error: Debug + Clone,
{
    pub fn is_loading(&self) -> bool {
        matches!(self, GenricAsyncState::Loading { .. })
    }
}
