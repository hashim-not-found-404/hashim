use crate::accounting_client::cache_op;
use crate::accounting_client::client_domain::cache;
use crate::accounting_client::client_domain::cache_actor;
use crate::accounting_client::client_domain::client_traits;
use crate::accounting_client::client_domain::process_manager;
use crate::accounting_client::client_domain::ui_model;
use crate::accounting_client::network_actor;
use crate::accounting_client::ui_effect;
use crate::accounting_domain::request_response;
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types;
use crate::utility::traits;
use crate::utility::traits::Receiver;
use crate::utility::traits::Sender;
use crate::utility::utils::ReadAndSet;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::RwLock;

pub fn new<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Ed: traits::Coding,
    Rg: traits::Regex,
    Ti: traits::Time,
    Ch: cache::Cache + 'static,
    Ws: network_actor::WSClient,
    As: ui_model::AllSignalTypes,
    Dbb: cache_op::DbBundle<Ch>,
>(
    model: &'static ui_model::Model<As>,
) -> ui_effect::Commander<Mpsc> {
    let (sender_to_network, receiver_to_network) = Mpsc::channel();
    let (sender_to_cache, receiver_to_cache) = Mpsc::channel();
    let (sender_to_error, receiver_to_error) = Mpsc::channel();

    let is_online = Arc::new(RwLock::new(false));

    network_actor::network_actor::<Rt, Ws, _>(
        MyNetwork::<Mpsc> {
            sender_to_cache: sender_to_cache.clone(),
            receiver_to_network,
            sender_to_error: sender_to_error.clone(),
            is_online: is_online.clone(),
        },
        format!("ws://{}/ws", types::ADDRESS),
    );

    let cache = client_traits::CacheActorStruct::new::<Rt, Ed, MyCache<Mpsc, Ch, Id, Ti, Dbb>>(
        receiver_to_cache,
        sender_to_cache,
        sender_to_network,
        sender_to_error,
        is_online,
    );

    let sender_to_process_manager = process_manager::process_manager_actor::<Mpsc, As, Rt>();

    ui_effect::Commander::new::<As, Rt, Rn, Id, Ti, Ch, Dbb>(
        receiver_to_error,
        sender_to_process_manager,
        model,
        cache,
    )
}

struct MyNetwork<Mpsc: traits::MultiProducerSingleConsumer> {
    sender_to_cache: Mpsc::Sender<
        cache_actor::MessageToCache<
            Mpsc,
            resource_utils::Subscribe,
            request_response::push_data::OperationsInput,
            request_response::push_data::OperationsResult,
        >,
    >,
    receiver_to_network: Mpsc::Receiver<Vec<u8>>,
    sender_to_error:     Mpsc::Sender<types::HashimError>,
    is_online:           Arc<RwLock<bool>>,
}

impl<Mpsc: traits::MultiProducerSingleConsumer> network_actor::Network for MyNetwork<Mpsc> {
    async fn network_state(&mut self, is_online: bool) {
        self.is_online.put(is_online);

        if is_online {
            self.sender_to_cache.send(cache_actor::MessageToCache::WeAreBackOnline).await.unwrap();
        }
    }

    async fn network_sender(&mut self, data: Vec<u8>) {
        self.sender_to_cache.send(cache_actor::MessageToCache::DataFromServer(data)).await.unwrap();
    }

    async fn network_reciever(&mut self) -> Vec<u8> {
        self.receiver_to_network.recv().await.unwrap()
    }

    async fn send_error(&mut self, _: traits::DynamicError) {
        self.sender_to_error.send(types::HashimError::ConnectionClosed).await.unwrap();
    }
}

struct MyCache<
    Mpsc: traits::MultiProducerSingleConsumer,
    Ch: cache::Cache,
    Id: types::RowId,
    Ti: traits::Time,
    Dbb: cache_op::DbBundle<Ch>,
> {
    _ph: PhantomData<(Mpsc, Ch, Id, Ti, Dbb)>,
}

impl<
    Mpsc: traits::MultiProducerSingleConsumer,
    Ch: cache::Cache,
    Id: types::RowId,
    Ti: traits::Time,
    Dbb: cache_op::DbBundle<Ch>,
> cache_actor::CacheActorUtils for MyCache<Mpsc, Ch, Id, Ti, Dbb>
{
    type Cache = Ch;
    type E = types::HashimError;
    type ErrorSender = Mpsc::Sender<types::HashimError>;
    type MessageFromServer<'de> = request_response::messages::FromServer;
    type Mpsc = Mpsc;
    type NetworkSender = Mpsc::Sender<Vec<u8>>;
    type NetworkStatus = Arc<RwLock<bool>>;
    type OpInput = request_response::push_data::OperationsInput;
    type OpResult = request_response::push_data::OperationsResult;
    type Resource = resource_utils::ResourceInfo;
    type Response = request_response::push_data::MyResult;
    type SendingTxns = request_response::messages::FromClient;
    type Subscribe = resource_utils::Subscribe;

    async fn cache_receiver(
        receiver: &mut <Self::Mpsc as traits::MultiProducerSingleConsumer>::Receiver<
            cache_actor::MessageToCache<Self::Mpsc, Self::Subscribe, Self::OpInput, Self::OpResult>,
        >,
    ) -> cache_actor::MessageToCache<Self::Mpsc, Self::Subscribe, Self::OpInput, Self::OpResult>
    {
        receiver.recv().await.unwrap()
    }

    async fn send_to_network(sender: &mut Self::NetworkSender, data: Vec<u8>) {
        sender.send(data).await.unwrap();
    }

    async fn internal_server_error(sender: &mut Self::ErrorSender) {
        sender.send(types::HashimError::InternalServerError).await.unwrap();
    }

    async fn invalid_format_error(sender: &mut Self::ErrorSender) {
        sender.send(types::HashimError::InvalidDataFormat).await.unwrap();
    }

    async fn is_online(network_status: &Self::NetworkStatus) -> bool {
        network_status.read()
    }

    async fn new_cache() -> Self::Cache {
        cache_op::new::<Id, Ti, Dbb, Ch>().await
    }

    async fn get_all_pending_txn(cache: &Self::Cache) -> Vec<(u64, Self::OpInput)> {
        let txns = cache.get_all_txn_input().await;

        let mut result = Vec::with_capacity(txns.len());
        for txn in txns {
            result.push((txn.txn_number, txn.operation));
        }
        result
    }

    async fn clear_state_pending_txn(cache: &mut Self::Cache) {
        cache.clear_pending_txn_state().await;
    }

    async fn start_state_pending_txn(cache: &mut Self::Cache) {
        cache.start_pending_txn_state().await;
    }

    async fn prepare_txn_for_send(
        cache: &Self::Cache,
        txns: Vec<(u64, Self::OpInput)>,
    ) -> Self::SendingTxns {
        let mut jwts = Vec::new();
        for (_, txn) in &txns {
            if let Some(user_uuid) = txn.get_user_uuid::<Ti, Ch, Dbb>()
                && let Some(jwt) = cache.get_jwt(user_uuid).await
            {
                jwts.push(jwt)
            }
        }

        let mut operations1 = Vec::with_capacity(txns.len());

        for i in txns {
            operations1.push(request_response::push_data::Txn {
                txn_number: i.0,
                operation:  i.1,
            });
        }

        request_response::messages::FromClient {
            jwts,
            nonce: Id::generate(),
            operations: operations1,
        }
    }

    fn to(
        msg: Self::MessageFromServer<'_>,
    ) -> cache_actor::FromServer<Self::E, Self::Response, Self::Resource> {
        match msg {
            request_response::messages::FromServer::Error(a) => cache_actor::FromServer::Error(a),
            request_response::messages::FromServer::PushData(a) => {
                cache_actor::FromServer::Response(a)
            }
            request_response::messages::FromServer::Resources(a) => {
                cache_actor::FromServer::Resources(a)
            }
        }
    }

    fn extract_resource(resp: &Self::Response) -> Vec<Self::Resource> {
        resp.operations.iter().flat_map(|a| a.operation.extract_resource::<Ti, Ch, Dbb>()).collect()
    }

    async fn write_resource(cache: &Self::Cache, resource: &[Self::Resource]) {
        cache.write_resource_from_server(resource).await;
    }

    async fn delete_successful_txn_input(cache: &Self::Cache, resp: &Self::Response) {
        for i in &resp.operations {
            if i.operation.is_ok() {
                cache.delete_txn_input(&i.txn_number).await;
            }
        }
    }

    async fn mark_txn_input_as_faild(cache: &Self::Cache, resp: &Self::Response) {
        for i in &resp.operations {
            if !i.operation.is_ok() {
                cache.mark_txn_input_as_faild(&i.txn_number).await;
            }
        }
    }

    async fn write_faild_txn_result(cache: &Self::Cache, resp: &Self::Response) {
        for i in &resp.operations {
            if !i.operation.is_ok() {
                cache.write_txn_result(i).await;
            }
        }
    }

    async fn get_all_response_txn_numbers(resp: &Self::Response) -> Vec<(u64, Self::OpResult)> {
        resp.operations.iter().map(|a| (a.txn_number, a.operation.clone())).collect()
    }

    fn collect_subs_to_poke(
        subs_to_poke: &mut std::collections::HashSet<Self::Subscribe>,
        resource: &[Self::Resource],
    ) {
        for i in resource {
            if let Some(value) = i.resource.map_to_subs() {
                subs_to_poke.insert(value);
            }
        }
    }

    async fn check_input(cache: &mut Self::Cache, data: &Self::OpInput) -> Self::OpResult {
        data.run_operation_check::<Id, Ti, Ch, Dbb>(cache).await
    }

    fn extract_resource1(data: &Self::OpResult) -> Vec<Self::Resource> {
        data.extract_resource::<Ti, Ch, Dbb>()
    }

    async fn apply_input(cache: &mut Self::Cache, resource: &[Self::Resource]) {
        cache.write_resource_of_pending_txn(resource).await;
    }

    async fn write_input(cache: &Self::Cache, txn_number: u64, data: &Self::OpInput) {
        cache
            .write_txn_input(&request_response::push_data::Txn {
                txn_number,
                operation: data.clone(),
            })
            .await;
    }
}
