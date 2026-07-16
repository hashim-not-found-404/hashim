use crate::{
    accounting_client::{
        network_actor, ui_effect,
        use_cases::client_domain::{cache, cache_actor, client_traits, process_manager, ui_model},
    },
    accounting_domain::{cases, request_response, types},
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
    Id: cases::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Ed: traits::Coding,
    Rg: traits::Regex,
    Ch: cache::Cache,
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

    let cache = client_traits::CacheActorStruct::new::<Rt, Ed, Rn, MyCache<Mpsc>>(
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
            types::Subscribe,
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

    async fn send_error(&mut self, error: traits::DynamicError) {
        self.sender_to_error
            .send(types::HashimError::ConnectionClosed)
            .await
            .unwrap();
    }
}

struct MyCache<Mpsc: traits::MultiProducerSingleConsumer> {
    _ph: PhantomData<Mpsc>,
}

impl<Mpsc: traits::MultiProducerSingleConsumer> cache_actor::CacheActorUtils for MyCache<Mpsc> {
    type Mpsc = Mpsc;
    type Subscribe = types::Subscribe;
    type OpInput = request_response::push_data::OperationsInput;
    type OpResult = request_response::push_data::OperationsResult;
    async fn cache_receiver(
        receiver: &mut <Self::Mpsc as traits::MultiProducerSingleConsumer>::Receiver<
            cache_actor::MessageToCache<Self::Mpsc, Self::Subscribe, Self::OpInput, Self::OpResult>,
        >,
    ) -> cache_actor::MessageToCache<Self::Mpsc, Self::Subscribe, Self::OpInput, Self::OpResult>
    {
        todo!()
    }

    type NetworkSender = Mpsc::Sender<Vec<u8>>;
    async fn send_to_network(sender: &mut Self::NetworkSender, data: Vec<u8>) {
        todo!()
    }

    type ErrorSender = Mpsc::Sender<types::HashimError>;
    async fn internal_server_error(sender: &mut Self::ErrorSender) {
        todo!()
    }

    async fn invalid_format_error(sender: &mut Self::ErrorSender) {
        todo!()
    }

    type NetworkStatus = Arc<RwLock<bool>>;
    async fn is_online(network_status: &Self::NetworkStatus) -> bool {
        todo!()
    }

    type Cache = bool;
    async fn new_cache() -> Self::Cache {
        todo!()
    }

    async fn get_all_pending_txn(cache: &Self::Cache) -> Vec<(u64, Self::OpInput)> {
        todo!()
    }

    async fn clear_state_pending_txn(cache: &Self::Cache) {
        todo!()
    }

    type SendingTxns = bool;
    async fn prepare_txn_for_send(
        cache: &Self::Cache,
        txns: Vec<(u64, Self::OpInput)>,
    ) -> Self::SendingTxns {
        todo!()
    }

    type E = bool;
    type Response = bool;
    type Resource = bool;
    type MessageFromServer<'de> = bool;
    fn to<'de>(
        msg: Self::MessageFromServer<'de>,
    ) -> cache_actor::FromServer<Self::E, Self::Response, Self::Resource> {
        todo!()
    }

    fn extract_resource(resp: &Self::Response) -> Vec<Self::Resource> {
        todo!()
    }

    async fn write_resource(cache: &Self::Cache, resource: &Vec<Self::Resource>) {
        todo!()
    }

    async fn delete_successful_txn_input(cache: &Self::Cache, resp: &Self::Response) {
        todo!()
    }

    async fn mark_txn_input_as_faild(cache: &Self::Cache, resp: &Self::Response) {
        todo!()
    }

    async fn write_faild_txn_result(cache: &Self::Cache, resp: &Self::Response) {
        todo!()
    }

    async fn get_all_response_txn_numbers(resp: &Self::Response) -> Vec<(u64, Self::OpResult)> {
        todo!()
    }

    fn collect_subs_to_poke(
        subs_to_poke: &mut std::collections::HashSet<Self::Subscribe>,
        resource: &Vec<Self::Resource>,
    ) {
        todo!()
    }

    async fn check_input(
        cache: &Self::Cache,
        data: &Self::OpInput,
    ) -> (Self::OpResult, Vec<Self::Resource>) {
        todo!()
    }

    async fn apply_input(cache: &Self::Cache, resource: &Vec<Self::Resource>) {
        todo!()
    }

    async fn write_input(cache: &Self::Cache, data: &Self::OpInput) {
        todo!()
    }
}
