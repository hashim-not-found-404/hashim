use crate::accounting_client::client_domain::cache::Cache;
use crate::accounting_client::client_domain::cache_actor::CacheStruct;
use crate::accounting_client::client_domain::cache_actor::CachingStrategy;
use crate::accounting_client::client_domain::cache_actor::Response;
use crate::accounting_client::client_domain::commander::CommanderLocalState;
use crate::accounting_client::client_domain::process_manager::MessageFromProcess;
use crate::accounting_client::client_domain::process_manager::MessageToProcess;
use crate::accounting_client::client_domain::process_manager::MessageToProcessManager;
use crate::accounting_client::client_domain::process_manager::ProcessName;
use crate::accounting_client::client_domain::ui_model::AllSignalTypes;
use crate::accounting_client::client_domain::ui_model::Model;
use crate::accounting_domain::request_response::OperationsInput;
use crate::accounting_domain::request_response::OperationsResult;
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::resource_utils::Subscribe;
use crate::accounting_domain::utility::types::RowId;
use crate::accounting_domain::utility::uuid::User;
use crate::utility::traits::JoinHandle;
use crate::utility::traits::MultiProducerSingleConsumer;
use crate::utility::traits::RandomNumber;
use crate::utility::traits::Receiver;
use crate::utility::traits::Runtime;
use crate::utility::traits::Sender;
use crate::utility::utils::ReadAndSet;
use std::sync::Arc;

pub(crate) trait ViewAndCache<Ch: Cache, T> {
    type Type1;
    type Type2;
    type Type3;
    type Type4;

    fn subs() -> &'static [Subscribe] {
        unreachable!("we dont need it here")
    }

    fn wrap_input(data: Self::Type1) -> OperationsInput;
    fn user_uuid(data: &Self::Type2) -> Option<&User>;
    async fn state_full_operation<Id: RowId>(data: &Self::Type2, state: &mut Ch) -> Self::Type3;
    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo>;
    fn unwrap_output(output: OperationsResult) -> Self::Type4;
    fn apply_on_the_model<As: AllSignalTypes>(output: &Self::Type4, model: &Model<As>);
}

pub(crate) trait ReadServerOnly {
    type Type1;
    type Type2;
    type Type3;

    fn wrap_input(data: Self::Type1) -> OperationsInput;
    fn user_uuid(data: &Self::Type2) -> Option<&User>;
    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo>;
}

pub(crate) type CacheActorStruct<Mpsc> =
    CacheStruct<Mpsc, Subscribe, OperationsInput, OperationsResult>;

pub(crate) async fn handle_fall_back<
    Rn: RandomNumber,
    Rt: Runtime,
    Mpsc: MultiProducerSingleConsumer,
    As: AllSignalTypes,
>(
    mut cache: CacheActorStruct<Mpsc>,
    commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
    dialog: &'static As::Dialog,
    process_name: ProcessName,
    data: OperationsInput,
    f: impl Fn(OperationsResult) -> bool + Clone + 'static,
) {
    let txn_number = Rn::generate();

    let f1 = f.clone();
    let data1 = data.clone();
    let mut cache1 = cache.clone();
    let commander_local_state1 = commander_local_state.clone();

    let mut handle = <Rt>::abortable_spawn_local(async move {
        let mut receiver_to_response =
            cache1.send_to_cache_actor(CachingStrategy::WriteServerOnly, txn_number, data1).await;

        match receiver_to_response.recv().await.unwrap() {
            Response::CloseTheChannel => {}
            Response::ServerCannotBeReached => {}
            Response::Data {
                is_response_from_server,
                data,
            } => {
                commander_local_state1
                    .sender_to_process_manager
                    .read()
                    .send(MessageToProcessManager::FromProcess {
                        process_name,
                        message: MessageFromProcess::Response {
                            is_response_from_server,
                            is_response_ok: f1(data),
                        },
                    })
                    .await
                    .unwrap();
            }
        }
    });

    let (sender, mut receiver_to_process) = <Mpsc>::channel();
    commander_local_state
        .sender_to_process_manager
        .read()
        .send(MessageToProcessManager::FromProcess {
            process_name,
            message: MessageFromProcess::Subscribe {
                sender,
                dialog,
            },
        })
        .await
        .unwrap();

    match receiver_to_process.recv().await.unwrap() {
        MessageToProcess::CancelOperation => {}
        MessageToProcess::FallBackToCache => {
            let mut receiver_to_response =
                cache.send_to_cache_actor(CachingStrategy::WriteCacheOnly, txn_number, data).await;

            match receiver_to_response.recv().await.unwrap() {
                Response::CloseTheChannel => {}
                Response::ServerCannotBeReached => {}
                Response::Data {
                    is_response_from_server: _,
                    data,
                } => {
                    f(data);
                }
            }
        }
    }
    handle.abort().await;
}

pub(crate) fn spawn_listener<Rn: RandomNumber, Rt: Runtime, Mpsc: MultiProducerSingleConsumer>(
    mut cache: CacheActorStruct<Mpsc>,
    list_of_subscribtion: &'static [Subscribe],
    data: OperationsInput,
    is_error: impl Fn(OperationsResult) + 'static,
) -> impl FnOnce() {
    let component_id = Rn::generate() as u16;
    let mut cache1 = cache.clone();

    let mut handle = Rt::abortable_spawn_local(async move {
        let mut receiver_to_poke =
            cache.send_subs_to_cache_actor(component_id, list_of_subscribtion).await;

        cache
            .send_to_cache_actor(CachingStrategy::ReadServerOnly, Rn::generate(), data.clone())
            .await;

        loop {
            let value = cache
                .send_to_cache_actor(CachingStrategy::ReadCacheOnly, Rn::generate(), data.clone())
                .await
                .recv()
                .await
                .unwrap();

            if let Response::Data {
                data,
                ..
            } = value
            {
                is_error(data);
            };

            if receiver_to_poke.recv().await.is_err() {
                break;
            }
        }
        cache.send_unsubs_to_cache_actor(component_id).await;
    });

    move || {
        Rt::spawn_local(async move {
            handle.abort().await;
            cache1.send_unsubs_to_cache_actor(component_id).await;
        });
    }
}
