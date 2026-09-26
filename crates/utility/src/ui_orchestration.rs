use crate::cache::CacheStruct;
use crate::cache::CachingStrategy;
use crate::cache::Response;
use crate::cache::Subscribe;
use crate::cache::TypeOperationClientInput;
use crate::cache::TypeOperationClientResult;
use crate::dtos::TxnNumber;
use crate::handle_errors::handle_error_one_time;
use crate::process_manager::DialogType;
use crate::process_manager::MessageFromProcess;
use crate::process_manager::MessageToProcess;
use crate::process_manager::MessageToProcessManager;
use crate::process_manager::ProcessId;
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

pub fn spawn_listener(
    sender_to_error: MpscSender<Error>,
    mut cache: CacheStruct,
    list_of_subscribtion: &'static [Subscribe],
    data: TypeOperationClientInput,
    is_error: impl Fn(TypeOperationClientResult) + 'static,
) -> impl FnOnce() {
    let component_id = Rn::generate() as u16;
    let mut cache1 = cache.clone();
    let sender_to_error1 = sender_to_error.clone();

    let mut handle = Rt::abortable_spawn_local(async move {
        handle_error_one_time(sender_to_error1, async move || {
            let mut receiver_to_poke = cache
                .send_subs_to_cache_actor(component_id, list_of_subscribtion)
                .await?;

            cache
                .send_to_cache_actor(
                    CachingStrategy::ReadServerOnly,
                    TxnNumber::default(),
                    data.clone(),
                )
                .await?;

            loop {
                let value = cache
                    .send_to_cache_actor(
                        CachingStrategy::ReadCacheOnly,
                        TxnNumber::default(),
                        data.clone(),
                    )
                    .await?
                    .recv()
                    .await?;

                if let Response::Data { data, .. } = value {
                    is_error(data);
                }

                if receiver_to_poke.recv().await.is_err() {
                    break;
                }
            }
            cache.send_unsubs_to_cache_actor(component_id).await?;

            Ok(())
        })
        .await;
    });

    move || {
        Rt::spawn_local(async move {
            handle_error_one_time(sender_to_error, async move || {
                handle.abort().await;
                cache1.send_unsubs_to_cache_actor(component_id).await?;
                Ok(())
            })
            .await;
        });
    }
}
