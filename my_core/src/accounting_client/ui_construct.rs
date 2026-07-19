use crate::{
    accounting_client::{
        network_actor, ui_effect,
        use_cases::client_domain::{cache, cache_actor, client_traits, process_manager, ui_model},
    },
    accounting_domain::{
        cases::utility::{resource_utils, types},
        request_response,
    },
    utility::{
        traits::{self, Receiver, Sender},
        utils::ReadAndSet,
    },
};
use std::{
    marker::PhantomData,
    sync::{Arc, RwLock},
};

pub fn new<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Ed: traits::Coding,
    Rg: traits::Regex,
    Ch: cache::Cache + 'static,
    Ws: network_actor::WSClient,
    As: ui_model::AllSignalTypes,
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
            receiver_to_network: receiver_to_network,
            sender_to_error: sender_to_error.clone(),
            is_online: is_online.clone(),
        },
        format!("ws://{}/ws", types::ADDRESS),
    );

    let cache = client_traits::CacheActorStruct::new::<Rt, Ed, Rn, MyCache<Mpsc, Ch, Id>>(
        receiver_to_cache,
        sender_to_cache,
        sender_to_network,
        sender_to_error,
        is_online,
    );

    let sender_to_process_manager = process_manager::process_manager_actor::<Mpsc, As, Rt>();

    let commander = ui_effect::Commander::new::<As, Rt, Rn, Id, Rg>(
        receiver_to_error,
        sender_to_process_manager,
        model,
        cache,
    );

    commander
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
    sender_to_error: Mpsc::Sender<types::HashimError>,
    is_online: Arc<RwLock<bool>>,
}

impl<Mpsc: traits::MultiProducerSingleConsumer> network_actor::Network for MyNetwork<Mpsc> {
    async fn network_state(&mut self, is_online: bool) {
        self.is_online.put(is_online);

        if is_online {
            self.sender_to_cache
                .send(cache_actor::MessageToCache::WeAreBackOnline)
                .await
                .unwrap();
        }
    }

    async fn network_sender(&mut self, data: Vec<u8>) {
        self.sender_to_cache
            .send(cache_actor::MessageToCache::DataFromServer(data))
            .await
            .unwrap();
    }

    async fn network_reciever(&mut self) -> Vec<u8> {
        self.receiver_to_network.recv().await.unwrap()
    }

    async fn send_error(&mut self, _: traits::DynamicError) {
        self.sender_to_error
            .send(types::HashimError::ConnectionClosed)
            .await
            .unwrap();
    }
}

struct MyCache<Mpsc: traits::MultiProducerSingleConsumer, Ch: cache::Cache, Id: types::RowId> {
    _ph: PhantomData<(Mpsc, Ch, Id)>,
}

impl<Mpsc: traits::MultiProducerSingleConsumer, Ch: cache::Cache, Id: types::RowId>
    cache_actor::CacheActorUtils for MyCache<Mpsc, Ch, Id>
{
    type Mpsc = Mpsc;
    type Subscribe = resource_utils::Subscribe;
    type OpInput = request_response::push_data::OperationsInput;
    type OpResult = request_response::push_data::OperationsResult;
    async fn cache_receiver(
        receiver: &mut <Self::Mpsc as traits::MultiProducerSingleConsumer>::Receiver<
            cache_actor::MessageToCache<Self::Mpsc, Self::Subscribe, Self::OpInput, Self::OpResult>,
        >,
    ) -> cache_actor::MessageToCache<Self::Mpsc, Self::Subscribe, Self::OpInput, Self::OpResult>
    {
        receiver.recv().await.unwrap()
    }

    type NetworkSender = Mpsc::Sender<Vec<u8>>;
    async fn send_to_network(sender: &mut Self::NetworkSender, data: Vec<u8>) {
        sender.send(data).await.unwrap();
    }

    type ErrorSender = Mpsc::Sender<types::HashimError>;
    async fn internal_server_error(sender: &mut Self::ErrorSender) {
        sender
            .send(types::HashimError::InternalServerError)
            .await
            .unwrap();
    }

    async fn invalid_format_error(sender: &mut Self::ErrorSender) {
        sender
            .send(types::HashimError::InvalidDataFormat)
            .await
            .unwrap();
    }

    type NetworkStatus = Arc<RwLock<bool>>;
    async fn is_online(network_status: &Self::NetworkStatus) -> bool {
        network_status.read()
    }

    type Cache = cache::State<Ch>;
    async fn new_cache() -> Self::Cache {
        cache::State::<Ch>::new::<Id>().await
    }

    async fn get_all_pending_txn(cache: &Self::Cache) -> Vec<(u64, Self::OpInput)> {
        let txns = cache.cache.get_all_txn_input().await;

        let mut result = Vec::with_capacity(txns.len());
        for txn in txns {
            result.push((txn.txn_number, txn.operation));
        }
        result
    }

    async fn clear_state_pending_txn(cache: &mut Self::Cache) {
        cache.state_of_pending_txn = Default::default();
    }

    type SendingTxns = request_response::messages::FromClient;
    async fn prepare_txn_for_send(
        cache: &Self::Cache,
        txns: Vec<(u64, Self::OpInput)>,
    ) -> Self::SendingTxns {
        let mut jwts = Vec::new();
        for (_, txn) in &txns {
            if let Some(user_uuid) = txn.get_user_uuid() {
                if let Some(jwt) = cache.cache.get_jwt(user_uuid).await {
                    jwts.push(jwt)
                }
            }
        }

        let mut operations1 = Vec::with_capacity(txns.len());

        for i in txns {
            operations1.push(request_response::push_data::Txn {
                txn_number: i.0,
                operation: i.1,
            });
        }

        let t = request_response::messages::FromClient {
            jwts,
            nonce: Id::generate(),
            operations: operations1,
        };

        t
    }

    type E = types::HashimError;
    type Response = request_response::push_data::MyResult;
    type Resource = resource_utils::ResourceInfo;
    type MessageFromServer<'de> = request_response::messages::FromServer;
    fn to<'de>(
        msg: Self::MessageFromServer<'de>,
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
        resp.operations
            .iter()
            .flat_map(|a| a.operation.extract_resource())
            .collect()
    }

    async fn write_resource(cache: &Self::Cache, resource: &Vec<Self::Resource>) {
        cache.cache.write_resource(resource).await;
    }

    async fn delete_successful_txn_input(cache: &Self::Cache, resp: &Self::Response) {
        for i in &resp.operations {
            if i.operation.is_ok() {
                cache.cache.delete_txn_input(&i.txn_number).await;
            }
        }
    }

    async fn mark_txn_input_as_faild(cache: &Self::Cache, resp: &Self::Response) {
        for i in &resp.operations {
            if !i.operation.is_ok() {
                cache.cache.mark_txn_input_as_faild(&i.txn_number).await;
            }
        }
    }

    async fn write_faild_txn_result(cache: &Self::Cache, resp: &Self::Response) {
        for i in &resp.operations {
            if !i.operation.is_ok() {
                cache.cache.write_txn_result(i).await;
            }
        }
    }

    async fn get_all_response_txn_numbers(resp: &Self::Response) -> Vec<(u64, Self::OpResult)> {
        resp.operations
            .iter()
            .map(|a| (a.txn_number, a.operation.clone()))
            .collect()
    }

    fn collect_subs_to_poke(
        subs_to_poke: &mut std::collections::HashSet<Self::Subscribe>,
        resource: &Vec<Self::Resource>,
    ) {
        for i in resource {
            if let Some(value) = i.resource.map_to_subs() {
                subs_to_poke.insert(value);
            }
        }
    }

    async fn check_input(cache: &mut Self::Cache, data: &Self::OpInput) -> Self::OpResult {
        data.run_operation_check::<Id, Ch>(cache).await
    }

    fn extract_resource1(data: &Self::OpResult) -> Vec<Self::Resource> {
        data.extract_resource()
    }

    fn apply_input(cache: &mut Self::Cache, resource: &Vec<Self::Resource>) {
        resource_utils::apply_change(resource.clone(), &mut cache.state_of_pending_txn);
    }

    async fn write_input(cache: &Self::Cache, txn_number: u64, data: &Self::OpInput) {
        cache
            .cache
            .write_txn_input(&request_response::push_data::Txn {
                txn_number,
                operation: data.clone(),
            })
            .await;
    }
}
