use crate::utility::traits;
use crate::utility::traits::MultiProducerSingleConsumer;
use crate::utility::traits::Sender;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;

pub(crate) enum FromServer<E, Resp, Reso> {
    Error(E),
    Response(Resp),
    Resources(Vec<Reso>),
}

#[derive(Clone)]
pub(crate) enum Response<OpResult: 'static> {
    CloseTheChannel,
    ServerCannotBeReached,
    Data {
        is_response_from_server: bool,
        data:                    OpResult,
    },
}

pub(crate) enum MessageToCache<
    Mpsc: traits::MultiProducerSingleConsumer,
    Subscribe: 'static + Hash + Eq + Clone,
    OpInput: 'static,
    OpResult: 'static,
> {
    WeAreBackOnline,
    DataFromServer(Vec<u8>),
    Subscribe {
        component_id:         u16,
        list_of_subscribtion: &'static [Subscribe],
        sender:               Mpsc::Sender<()>,
    },
    UnSubscribe {
        component_id: u16,
    },
    Query {
        strategy:   CachingStrategy,
        sender:     Mpsc::Sender<Response<OpResult>>,
        txn_number: u64,
        data:       OpInput,
    },
}

pub(crate) trait CacheActorUtils {
    type Mpsc: traits::MultiProducerSingleConsumer;
    type Subscribe: 'static + Hash + Eq + Clone;
    type OpInput;
    type OpResult;

    async fn cache_receiver(
        receiver: &mut <Self::Mpsc as MultiProducerSingleConsumer>::Receiver<
            MessageToCache<Self::Mpsc, Self::Subscribe, Self::OpInput, Self::OpResult>,
        >,
    ) -> MessageToCache<Self::Mpsc, Self::Subscribe, Self::OpInput, Self::OpResult>;

    type NetworkSender;
    async fn send_to_network(sender: &mut Self::NetworkSender, data: Vec<u8>);

    type ErrorSender;
    async fn internal_server_error(sender: &mut Self::ErrorSender);
    async fn invalid_format_error(sender: &mut Self::ErrorSender);

    type NetworkStatus;
    async fn is_online(network_status: &Self::NetworkStatus) -> bool;

    type Cache;
    async fn new_cache() -> Self::Cache;

    async fn get_all_pending_txn(cache: &Self::Cache) -> Vec<(u64, Self::OpInput)>;
    async fn clear_state_pending_txn(cache: &mut Self::Cache);
    async fn start_state_pending_txn(cache: &mut Self::Cache);

    type SendingTxns: serde::Serialize;
    async fn prepare_txn_for_send(
        cache: &Self::Cache,
        txns: Vec<(u64, Self::OpInput)>,
    ) -> Self::SendingTxns;

    type E;
    type Response;
    type Resource;
    type MessageFromServer<'de>: serde::Deserialize<'de>;

    fn to(msg: Self::MessageFromServer<'_>) -> FromServer<Self::E, Self::Response, Self::Resource>;
    fn extract_resource(resp: &Self::Response) -> Vec<Self::Resource>;
    async fn write_resource(cache: &Self::Cache, resource: &[Self::Resource]);

    async fn delete_successful_txn_input(cache: &Self::Cache, resp: &Self::Response);
    async fn mark_txn_input_as_faild(cache: &Self::Cache, resp: &Self::Response);
    async fn write_faild_txn_result(cache: &Self::Cache, resp: &Self::Response);
    async fn get_all_response_txn_numbers(resp: &Self::Response) -> Vec<(u64, Self::OpResult)>;
    fn create_pending_txn(txn_number: u64, data: Self::OpInput) -> (u64, Self::OpInput) {
        (txn_number, data)
    }
    fn collect_subs_to_poke(
        subs_to_poke: &mut HashSet<Self::Subscribe>,
        resource: &[Self::Resource],
    );
    async fn check_input(cache: &mut Self::Cache, data: &Self::OpInput) -> Self::OpResult;
    fn extract_resource1(data: &Self::OpResult) -> Vec<Self::Resource>;
    async fn apply_input(cache: &mut Self::Cache, resource: &[Self::Resource]);
    async fn write_input(cache: &Self::Cache, txn_number: u64, data: &Self::OpInput);
}

#[allow(dead_code)]
pub(crate) enum CachingStrategy {
    ReadCacheOnly,
    ReadCacheFirst,
    ReadCacheAndServer,
    ReadServerFirst,
    ReadServerOnly,
    WriteCacheOnly,
    WriteCacheFirst,
    WriteCacheAndServer,
    WriteServerFirst,
    WriteServerOnly,
}

pub(crate) struct CacheStruct<Mpsc, Subscribe, OpInput, OpResult>
where
    Mpsc: traits::MultiProducerSingleConsumer,
    Subscribe: 'static + Hash + Eq + Clone,
    OpInput: 'static,
    OpResult: 'static,
{
    sender: Mpsc::Sender<MessageToCache<Mpsc, Subscribe, OpInput, OpResult>>,
}

impl<Mpsc, Subscribe, OpInput, OpResult> Clone for CacheStruct<Mpsc, Subscribe, OpInput, OpResult>
where
    Mpsc: traits::MultiProducerSingleConsumer,
    Subscribe: 'static + Hash + Eq + Clone,
{
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<Mpsc, Subscribe, OpInput, OpResult> CacheStruct<Mpsc, Subscribe, OpInput, OpResult>
where
    Mpsc: traits::MultiProducerSingleConsumer,
    Subscribe: 'static + Hash + Eq + Clone,
{
    pub(crate) fn new<
        Rt: traits::Runtime,
        Ed: traits::Coding,
        Cu: CacheActorUtils<OpResult = OpResult, Subscribe = Subscribe, Mpsc = Mpsc> + 'static,
    >(
        receiver_to_cache: <Cu::Mpsc as MultiProducerSingleConsumer>::Receiver<
            MessageToCache<Cu::Mpsc, Cu::Subscribe, Cu::OpInput, Cu::OpResult>,
        >,
        sender_to_cache: Mpsc::Sender<MessageToCache<Mpsc, Subscribe, OpInput, OpResult>>,
        sender_to_network: Cu::NetworkSender,
        sender_to_error: Cu::ErrorSender,
        is_online: Cu::NetworkStatus,
    ) -> Self {
        Self::cache_actor::<Rt, Ed, Cu>(
            receiver_to_cache,
            sender_to_network,
            sender_to_error,
            is_online,
        );

        Self {
            sender: sender_to_cache,
        }
    }

    pub(crate) async fn send_to_cache_actor(
        &mut self,
        strategy: CachingStrategy,
        txn_number: u64,
        data: OpInput,
    ) -> Mpsc::Receiver<Response<OpResult>> {
        let (sender, receiver) = Mpsc::channel();

        self.sender
            .send(MessageToCache::Query {
                strategy,
                sender,
                txn_number,
                data,
            })
            .await
            .unwrap();

        receiver
    }

    pub(crate) async fn send_subs_to_cache_actor(
        &mut self,
        component_id: u16,
        list_of_subscribtion: &'static [Subscribe],
    ) -> Mpsc::Receiver<()> {
        let (sender, receiver) = Mpsc::channel();

        self.sender
            .send(MessageToCache::Subscribe {
                component_id,
                list_of_subscribtion,
                sender,
            })
            .await
            .unwrap();

        receiver
    }

    pub(crate) async fn send_unsubs_to_cache_actor(&mut self, component_id: u16) {
        self.sender
            .send(MessageToCache::UnSubscribe {
                component_id,
            })
            .await
            .unwrap();
    }

    fn cache_actor<
        Rt: traits::Runtime,
        Ed: traits::Coding,
        Cu: CacheActorUtils<OpResult = OpResult, Subscribe = Subscribe, Mpsc = Mpsc> + 'static,
    >(
        mut receiver_to_cache: <Cu::Mpsc as MultiProducerSingleConsumer>::Receiver<
            MessageToCache<Cu::Mpsc, Cu::Subscribe, Cu::OpInput, Cu::OpResult>,
        >,
        mut sender_to_network: Cu::NetworkSender,
        mut sender_to_error: Cu::ErrorSender,
        is_online: Cu::NetworkStatus,
    ) {
        Rt::spawn_local(async move {
            let mut pool_of_senders =
                HashMap::<u64, Mpsc::Sender<Response<OpResult>>>::with_capacity(100);
            let mut pool_of_pokers = HashMap::<u16, Mpsc::Sender<()>>::with_capacity(10);
            let mut pool_of_subscribes = HashMap::<Subscribe, HashSet<u16>>::with_capacity(100);

            let mut cache = Cu::new_cache().await;

            loop {
                match Cu::cache_receiver(&mut receiver_to_cache).await {
                    MessageToCache::WeAreBackOnline => {
                        let txns = Cu::get_all_pending_txn(&cache).await;
                        if txns.is_empty() {
                            continue;
                        }
                        let txns = Cu::prepare_txn_for_send(&cache, txns).await;
                        let txns = Ed::encode(&txns);
                        Cu::send_to_network(&mut sender_to_network, txns).await;
                    }
                    MessageToCache::DataFromServer(raw_data) => {
                        let message_type = match Ed::decode::<Cu::MessageFromServer<'_>>(&raw_data)
                        {
                            Ok(message_type) => Cu::to(message_type),
                            Err(_) => {
                                Cu::invalid_format_error(&mut sender_to_error).await;
                                continue;
                            }
                        };

                        match message_type {
                            FromServer::Error(_) => {
                                Cu::internal_server_error(&mut sender_to_error).await;
                            }
                            FromServer::Response(response) => {
                                let mut subs_to_poke = HashSet::new();

                                Cu::clear_state_pending_txn(&mut cache).await;
                                let resource = Cu::extract_resource(&response);
                                Cu::write_resource(&cache, &resource).await;
                                Cu::delete_successful_txn_input(&cache, &response).await;
                                Cu::mark_txn_input_as_faild(&cache, &response).await;
                                Cu::write_faild_txn_result(&cache, &response).await;
                                Cu::collect_subs_to_poke(&mut subs_to_poke, &resource);

                                let txn_numbers = Cu::get_all_response_txn_numbers(&response).await;
                                for (number, data) in txn_numbers {
                                    let sender = pool_of_senders.remove(&number);
                                    if let Some(mut sender) = sender {
                                        let _ = sender
                                            .send(Response::Data {
                                                is_response_from_server: true,
                                                data,
                                            })
                                            .await;
                                        let _ = sender.send(Response::CloseTheChannel).await;
                                    }
                                }

                                Cu::start_state_pending_txn(&mut cache).await;
                                let txns = Cu::get_all_pending_txn(&cache).await;

                                for (_, txn) in txns {
                                    let result = Cu::check_input(&mut cache, &txn).await;
                                    let resource = Cu::extract_resource1(&result);
                                    Cu::apply_input(&mut cache, &resource).await;
                                }

                                poke_the_subs::<Mpsc, Subscribe>(
                                    &mut pool_of_pokers,
                                    &pool_of_subscribes,
                                    &subs_to_poke,
                                )
                                .await;
                            }
                            FromServer::Resources(resource) => {
                                Cu::write_resource(&cache, &resource).await;

                                let mut subs_to_poke = HashSet::new();
                                Cu::collect_subs_to_poke(&mut subs_to_poke, &resource);
                                poke_the_subs::<Mpsc, Subscribe>(
                                    &mut pool_of_pokers,
                                    &pool_of_subscribes,
                                    &subs_to_poke,
                                )
                                .await;
                            }
                        }
                    }
                    MessageToCache::Subscribe {
                        component_id,
                        list_of_subscribtion,
                        sender,
                    } => {
                        pool_of_pokers.insert(component_id, sender);
                        for subscribe in list_of_subscribtion {
                            pool_of_subscribes
                                .entry(subscribe.clone())
                                .or_default()
                                .insert(component_id);
                        }
                    }
                    MessageToCache::UnSubscribe {
                        component_id,
                    } => {
                        pool_of_pokers.remove(&component_id);

                        for components in &mut pool_of_subscribes.values_mut() {
                            components.remove(&component_id);
                        }

                        pool_of_subscribes.retain(|_, components| !components.is_empty());
                    }
                    MessageToCache::Query {
                        strategy,
                        mut sender,
                        txn_number,
                        data,
                    } => {
                        match strategy {
                            CachingStrategy::ReadCacheOnly => {
                                let result = Cu::check_input(&mut cache, &data).await;
                                let _ = sender
                                    .send(Response::Data {
                                        is_response_from_server: false,
                                        data:                    result,
                                    })
                                    .await;
                                let _ = sender.send(Response::CloseTheChannel).await;
                            }
                            CachingStrategy::ReadCacheFirst => todo!(),
                            CachingStrategy::ReadCacheAndServer => {
                                let result = Cu::check_input(&mut cache, &data).await;

                                let _ = sender
                                    .send(Response::Data {
                                        is_response_from_server: false,
                                        data:                    result,
                                    })
                                    .await;

                                let operations = Cu::create_pending_txn(txn_number, data);

                                if Cu::is_online(&is_online).await {
                                    let txn_to_send =
                                        Cu::prepare_txn_for_send(&cache, vec![operations]).await;

                                    let data = Ed::encode(&txn_to_send);
                                    Cu::send_to_network(&mut sender_to_network, data).await;

                                    pool_of_senders.insert(txn_number, sender);
                                } else {
                                    let _ = sender.send(Response::ServerCannotBeReached).await;
                                    let _ = sender.send(Response::CloseTheChannel).await;
                                };
                            }
                            CachingStrategy::ReadServerFirst => todo!(),
                            CachingStrategy::ReadServerOnly => {
                                let operations = Cu::create_pending_txn(txn_number, data);

                                if Cu::is_online(&is_online).await {
                                    let txn_to_send =
                                        Cu::prepare_txn_for_send(&cache, vec![operations]).await;

                                    let data = Ed::encode(&txn_to_send);
                                    Cu::send_to_network(&mut sender_to_network, data).await;

                                    pool_of_senders.insert(txn_number, sender);
                                } else {
                                    let _ = sender.send(Response::ServerCannotBeReached).await;
                                    let _ = sender.send(Response::CloseTheChannel).await;
                                };
                            }
                            CachingStrategy::WriteCacheOnly => {
                                let result = Cu::check_input(&mut cache, &data).await;
                                let resource = Cu::extract_resource1(&result);

                                Cu::apply_input(&mut cache, &resource).await;
                                Cu::write_input(&cache, txn_number, &data).await;

                                let mut subs_to_poke = HashSet::new();

                                Cu::collect_subs_to_poke(&mut subs_to_poke, &resource);

                                poke_the_subs::<Mpsc, Subscribe>(
                                    &mut pool_of_pokers,
                                    &pool_of_subscribes,
                                    &subs_to_poke,
                                )
                                .await;

                                let _ = sender
                                    .send(Response::Data {
                                        is_response_from_server: false,
                                        data:                    result,
                                    })
                                    .await;

                                let _ = sender.send(Response::CloseTheChannel).await;
                            }
                            CachingStrategy::WriteCacheFirst => todo!(),
                            CachingStrategy::WriteCacheAndServer => {
                                let result = Cu::check_input(&mut cache, &data).await;
                                let resource = Cu::extract_resource1(&result);

                                Cu::apply_input(&mut cache, &resource).await;
                                Cu::write_input(&cache, txn_number, &data).await;

                                let mut subs_to_poke = HashSet::new();

                                Cu::collect_subs_to_poke(&mut subs_to_poke, &resource);

                                poke_the_subs::<Mpsc, Subscribe>(
                                    &mut pool_of_pokers,
                                    &pool_of_subscribes,
                                    &subs_to_poke,
                                )
                                .await;

                                let _ = sender
                                    .send(Response::Data {
                                        is_response_from_server: false,
                                        data:                    result,
                                    })
                                    .await;

                                let operations = Cu::create_pending_txn(txn_number, data);

                                if Cu::is_online(&is_online).await {
                                    let txn_to_send =
                                        Cu::prepare_txn_for_send(&cache, vec![operations]).await;

                                    let data = Ed::encode(&txn_to_send);
                                    Cu::send_to_network(&mut sender_to_network, data).await;

                                    pool_of_senders.insert(txn_number, sender);
                                } else {
                                    let _ = sender.send(Response::ServerCannotBeReached).await;
                                    let _ = sender.send(Response::CloseTheChannel).await;
                                };
                            }
                            CachingStrategy::WriteServerFirst => todo!(),
                            CachingStrategy::WriteServerOnly => {
                                let operations = Cu::create_pending_txn(txn_number, data);

                                if Cu::is_online(&is_online).await {
                                    let txn_to_send =
                                        Cu::prepare_txn_for_send(&cache, vec![operations]).await;

                                    let data = Ed::encode(&txn_to_send);
                                    Cu::send_to_network(&mut sender_to_network, data).await;

                                    pool_of_senders.insert(txn_number, sender);
                                } else {
                                    let _ = sender.send(Response::ServerCannotBeReached).await;
                                    let _ = sender.send(Response::CloseTheChannel).await;
                                };
                            }
                        }
                    }
                }
            }
        });
    }
}

async fn poke_the_subs<
    Mpsc: traits::MultiProducerSingleConsumer,
    Subscribe: 'static + Hash + Eq,
>(
    pool_of_pokers: &mut HashMap<u16, Mpsc::Sender<()>>,
    pool_of_subscribes: &HashMap<Subscribe, HashSet<u16>>,
    subs_to_poke: &HashSet<Subscribe>,
) {
    let mut components_to_poke = HashSet::new();

    for one_sub in subs_to_poke {
        let Some(a) = pool_of_subscribes.get(one_sub) else {
            continue;
        };

        for a in a {
            components_to_poke.insert(a);
        }
    }

    for i in components_to_poke {
        let sender = pool_of_pokers.get_mut(i).unwrap();
        let _ = sender.send(()).await;
    }
}
