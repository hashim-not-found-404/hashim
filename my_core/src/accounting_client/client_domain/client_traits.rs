use crate::accounting_client::client_domain::cache;
use crate::accounting_client::client_domain::cache_actor;
use crate::accounting_client::client_domain::commander;
use crate::accounting_client::client_domain::process_manager;
use crate::accounting_client::client_domain::ui_model;
use crate::accounting_domain::request_response;
use crate::accounting_domain::request_response::push_data::OperationsResult;
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types;
use crate::utility::traits;
use crate::utility::traits::JoinHandle;
use crate::utility::traits::Receiver;
use crate::utility::traits::Sender;
use crate::utility::utils::ReadAndSet;
use std::sync::Arc;

pub(crate) trait ViewAndCache<Ch: cache::Cache, T> {
    type Type1;
    type Type2;
    type Type3;
    type Type4;

    fn subs() -> &'static [resource_utils::Subscribe] {
        unreachable!("we dont need it here")
    }

    fn wrap_input(data: Self::Type1) -> request_response::push_data::OperationsInput;

    fn user_uuid(data: &Self::Type2) -> Option<&types::UuidType>;

    async fn state_full_operation<Id: types::RowId>(
        data: &Self::Type2,
        state: &mut cache::State<Ch>,
    ) -> Self::Type3;

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo>;

    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self::Type4;

    fn apply_on_the_model<As: ui_model::AllSignalTypes>(
        output: &Self::Type4,
        model: &ui_model::Model<As>,
    );
}

pub(crate) trait ReadServerOnly {
    type Type1;
    type Type2;
    type Type3;

    fn wrap_input(data: Self::Type1) -> request_response::push_data::OperationsInput;
    fn user_uuid(data: &Self::Type2) -> Option<&types::UuidType>;
    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo>;
}

pub(crate) type CacheActorStruct<Mpsc> = cache_actor::CacheStruct<
    Mpsc,
    resource_utils::Subscribe,
    request_response::push_data::OperationsInput,
    request_response::push_data::OperationsResult,
>;

pub(crate) async fn handle_fall_back<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
>(
    mut cache: CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    dialog: &'static As::Dialog,
    process_name: process_manager::ProcessName,
    data: request_response::push_data::OperationsInput,
    f: impl Fn(OperationsResult) -> bool + Clone + 'static,
) {
    let txn_number = Rn::generate();

    let f1 = f.clone();
    let data1 = data.clone();
    let mut cache1 = cache.clone();
    let commander_local_state1 = commander_local_state.clone();

    let mut handle = <Rt>::abortable_spawn_local(async move {
        let mut receiver_to_response = cache1
            .send_to_cache_actor(cache_actor::CachingStrategy::WriteServerOnly, txn_number, data1)
            .await;

        match receiver_to_response.recv().await.unwrap() {
            cache_actor::Response::CloseTheChannel => {}
            cache_actor::Response::ServerCannotBeReached => {}
            cache_actor::Response::Data {
                is_response_from_server,
                data,
            } => {
                commander_local_state1
                    .sender_to_process_manager
                    .read()
                    .send(process_manager::MessageToProcessManager::FromProcess {
                        process_name,
                        message: process_manager::MessageFromProcess::Response {
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
        .send(process_manager::MessageToProcessManager::FromProcess {
            process_name,
            message: process_manager::MessageFromProcess::Subscribe {
                sender,
                dialog,
            },
        })
        .await
        .unwrap();

    match receiver_to_process.recv().await.unwrap() {
        process_manager::MessageToProcess::CancelOperation => {}
        process_manager::MessageToProcess::FallBackToCache => {
            let mut receiver_to_response = cache
                .send_to_cache_actor(cache_actor::CachingStrategy::WriteCacheOnly, txn_number, data)
                .await;

            match receiver_to_response.recv().await.unwrap() {
                cache_actor::Response::CloseTheChannel => {}
                cache_actor::Response::ServerCannotBeReached => {}
                cache_actor::Response::Data {
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
