use crate::client::utility::cache::Cache;
use crate::client::utility::cache_actor::CacheStruct;
use crate::client::utility::cache_actor::CachingStrategy;
use crate::client::utility::cache_actor::Response;
use crate::client::utility::commander::CommanderLocalState;
use crate::client::utility::process_manager::MessageFromProcess;
use crate::client::utility::process_manager::MessageToProcess;
use crate::client::utility::process_manager::MessageToProcessManager;
use crate::client::utility::process_manager::ProcessName;
use crate::client::utility::ui_model::AllSignalTypes;
use crate::client::utility::ui_model::Model;
use crate::domain::request_response::OperationsInput;
use crate::domain::request_response::OperationsResult;
use crate::domain::utility::resource_utils;
use crate::domain::utility::resource_utils::Subscribe;
use crate::domain::utility::types::RowId;
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

    async fn state_full_operation<Id: RowId>(data: &Self::Type2, state: &mut Ch) -> Self::Type3;
    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo>;
    fn apply_on_the_model<As: AllSignalTypes>(output: &Self::Type4, model: &Model<As>);
}

#[macro_export]
macro_rules! make_wrap_unwrap {
    ($case_name:ident, $variant:ident) => {
        pub(crate) fn wrap_input(
            data: crate::domain::use_cases::$case_name::Input,
        ) -> crate::domain::request_response::OperationsInput {
            crate::domain::request_response::OperationsInput::$variant(data)
        }
        pub(crate) fn unwrap_output(
            output: crate::domain::request_response::OperationsResult,
        ) -> crate::domain::use_cases::$case_name::MyResult {
            if let crate::domain::request_response::OperationsResult::$variant(result) = output {
                return result;
            }
            unreachable!("{:?}", output)
        }
    };
}

#[macro_export]
macro_rules! make_user_uuid {
    ($case_name:ident) => {
        pub(crate) fn user_uuid(
            data: &crate::domain::use_cases::$case_name::Input,
        ) -> Option<&User> {
            Some(&data.user_uuid)
        }
    };
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
