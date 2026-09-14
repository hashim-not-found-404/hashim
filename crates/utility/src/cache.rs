use anyhow::Result;
use infrastructure::actors::Mpsc;
use infrastructure::actors::MpscReceiver;
use infrastructure::actors::MpscSender;
use infrastructure::actors::MultiProducerSingleConsumer;
use infrastructure::actors::Sender;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use std::any::Any;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Subscribe(u32);

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct TxnNumber(pub u64);

pub trait ActorIO {
    type Cache: Cache;

    fn cache_receiver(
        receiver: &mut MpscReceiver<MessageToCache<Self::Cache>>,
    ) -> impl Future<Output = MessageToCache<Self::Cache>>;

    type NetworkSender;
    fn send_to_network(sender: &mut Self::NetworkSender, data: Vec<u8>)
    -> impl Future<Output = ()>;

    type ErrorSender;
    fn send_error(sender: &mut Self::ErrorSender, err: anyhow::Error) -> impl Future<Output = ()>;

    type NetworkStatus;
    fn is_online(network_status: &Self::NetworkStatus) -> impl Future<Output = bool>;
}

pub trait Cache: Sized + 'static {
    fn new_cache() -> impl Future<Output = Self>;

    fn get_all_pending_txn(&mut self) -> impl Future<Output = Vec<(TxnNumber, OpInput<Self>)>>;
    fn clear_state_pending_txn(&mut self) -> impl Future<Output = ()>;
    fn start_state_pending_txn(&mut self) -> impl Future<Output = ()>;

    fn delete_successful_txn_input(&mut self, txn_number: TxnNumber) -> impl Future<Output = ()>;
    fn mark_txn_input_as_faild(&mut self, txn_number: TxnNumber) -> impl Future<Output = ()>;

    fn write_input_to_cache(
        &mut self,
        txn_number: TxnNumber,
        input: OpInput<Self>,
    ) -> impl Future<Output = ()>;

    fn write_error_to_cache(
        &mut self,
        txn_number: TxnNumber,
        input: OpErr,
    ) -> impl Future<Output = ()>;

    fn encode_the_inputs(
        &mut self,
        inputs: Vec<(TxnNumber, OpInput<Self>)>,
    ) -> impl Future<Output = Vec<u8>>;

    fn decode_the_response(resp: Vec<u8>) -> Result<MessageFromServer<Self>>;
}

pub trait OpInputTrait: Debug {
    type Cache: Cache;

    fn check_input(
        &self,
        cache: &mut Self::Cache,
    ) -> Pin<Box<dyn Future<Output = OpResult<Self::Cache>>>>;
}

pub trait OpOkTrait: Debug {
    type Cache: Cache;

    fn apply_to_cache(&self, cache: &mut Self::Cache) -> Pin<Box<dyn Future<Output = ()>>>;
    fn subs_to_poke(&self) -> &'static [Subscribe];
}

pub trait OpErrTrait: Debug {}

pub trait OpResultTrait: Debug {
    type Cache: Cache;
    fn into_any(self: Arc<Self>) -> Box<dyn Any>;
    fn subs_to_poke(&self) -> &'static [Subscribe];
    fn extract_resource(&self) -> Result<OpOk<Self::Cache>, OpErr>;
}

pub trait TxnResultFromServer {
    type Cache: Cache;

    fn get_all_response_txn_numbers(
        &self,
    ) -> impl Future<Output = Vec<(TxnNumber, OpResult<Self::Cache>)>>;
}

#[derive(Debug)]
pub struct OpInput<Ch: Cache>(Arc<dyn OpInputTrait<Cache = Ch>>);
#[derive(Debug, Clone)]
pub struct OpOk<Ch: Cache>(Arc<dyn OpOkTrait<Cache = Ch>>);
#[derive(Debug, Clone)]
pub struct OpErr(Arc<dyn OpErrTrait>);
#[derive(Debug, Clone)]
pub struct OpResult<Ch: Cache>(Arc<dyn OpResultTrait<Cache = Ch>>);

impl<Ch: Cache, T: OpInputTrait<Cache = Ch> + 'static> From<T> for OpInput<Ch> {
    fn from(value: T) -> Self {
        Self(Arc::new(value))
    }
}

impl<Ch: Cache> Clone for OpInput<Ch> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Ch: Cache> OpResult<Ch> {
    pub fn downcast<T: 'static>(self) -> T {
        *self.0.into_any().downcast::<T>().unwrap()
    }
}

pub enum MessageFromServer<Ch: Cache> {
    Error(anyhow::Error),
    Response(Vec<(TxnNumber, OpResult<Ch>)>),
    Resources(Vec<OpOk<Ch>>),
}

#[derive(Debug, Clone)]
pub enum Response<Ch: Cache> {
    CloseTheChannel,
    ServerCannotBeReached,
    Data {
        is_response_from_server: bool,
        data:                    OpResult<Ch>,
    },
}

pub enum MessageToCache<Ch: Cache> {
    WeAreBackOnline,
    DataFromServer(Vec<u8>),
    Subscribe {
        component_id:         u16,
        list_of_subscribtion: &'static [Subscribe],
        sender:               MpscSender<()>,
    },
    UnSubscribe {
        component_id: u16,
    },
    Query {
        strategy:   CachingStrategy,
        sender:     MpscSender<Response<Ch>>,
        txn_number: TxnNumber,
        data:       OpInput<Ch>,
    },
}

#[allow(dead_code)]
pub enum CachingStrategy {
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

pub struct CacheStruct<Ch: Cache> {
    sender: MpscSender<MessageToCache<Ch>>,
}

impl<Ch: Cache> Clone for CacheStruct<Ch> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<Ch: Cache> CacheStruct<Ch> {
    pub fn new<Aio: ActorIO<Cache = Ch> + 'static>(
        receiver_to_cache: MpscReceiver<MessageToCache<Ch>>,
        sender_to_cache: MpscSender<MessageToCache<Ch>>,
        sender_to_network: Aio::NetworkSender,
        sender_to_error: Aio::ErrorSender,
        is_online: Aio::NetworkStatus,
    ) -> Self {
        Self::cache_actor::<Aio>(receiver_to_cache, sender_to_network, sender_to_error, is_online);

        Self {
            sender: sender_to_cache,
        }
    }

    pub async fn send_to_cache_actor(
        &mut self,
        strategy: CachingStrategy,
        txn_number: TxnNumber,
        data: OpInput<Ch>,
    ) -> MpscReceiver<Response<Ch>> {
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

    pub async fn send_subs_to_cache_actor(
        &mut self,
        component_id: u16,
        list_of_subscribtion: &'static [Subscribe],
    ) -> MpscReceiver<()> {
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

    pub async fn send_unsubs_to_cache_actor(&mut self, component_id: u16) {
        self.sender
            .send(MessageToCache::UnSubscribe {
                component_id,
            })
            .await
            .unwrap();
    }

    fn cache_actor<Aio: ActorIO<Cache = Ch> + 'static>(
        mut receiver_to_cache: MpscReceiver<MessageToCache<Ch>>,
        mut sender_to_network: Aio::NetworkSender,
        mut sender_to_error: Aio::ErrorSender,
        is_online: Aio::NetworkStatus,
    ) {
        Rt::spawn_local(async move {
            let mut pool_of_senders =
                HashMap::<TxnNumber, MpscSender<Response<Ch>>>::with_capacity(100);
            let mut pool_of_pokers = HashMap::<u16, MpscSender<()>>::with_capacity(10);
            let mut pool_of_subscribes = HashMap::<Subscribe, HashSet<u16>>::with_capacity(100);

            let mut cache = Ch::new_cache().await;

            loop {
                match Aio::cache_receiver(&mut receiver_to_cache).await {
                    MessageToCache::WeAreBackOnline => {
                        let txns = cache.get_all_pending_txn().await;
                        if txns.is_empty() {
                            continue;
                        }
                        let txns = cache.encode_the_inputs(txns).await;
                        Aio::send_to_network(&mut sender_to_network, txns).await;
                    }
                    MessageToCache::DataFromServer(raw_data) => {
                        let message_type = match Ch::decode_the_response(raw_data) {
                            Ok(ok) => ok,
                            Err(err) => {
                                Aio::send_error(&mut sender_to_error, err).await;
                                continue;
                            }
                        };

                        match message_type {
                            MessageFromServer::Error(err) => {
                                Aio::send_error(&mut sender_to_error, err).await;
                            }
                            MessageFromServer::Response(response) => {
                                let mut subs_to_poke = HashSet::new();

                                cache.clear_state_pending_txn().await;

                                for (txn_number, result) in response {
                                    cache.delete_successful_txn_input(txn_number).await;

                                    add_subs(&mut subs_to_poke, result.0.subs_to_poke());

                                    let resource = result.0.extract_resource();
                                    match resource {
                                        Ok(ok) => {
                                            ok.0.apply_to_cache(&mut cache).await;
                                            cache.delete_successful_txn_input(txn_number).await;
                                        }
                                        Err(err) => {
                                            cache.mark_txn_input_as_faild(txn_number).await;
                                            cache.write_error_to_cache(txn_number, err).await;
                                        }
                                    }

                                    let sender = pool_of_senders.remove(&txn_number);
                                    if let Some(mut sender) = sender {
                                        let _ = sender
                                            .send(Response::Data {
                                                is_response_from_server: true,
                                                data:                    result,
                                            })
                                            .await;
                                        let _ = sender.send(Response::CloseTheChannel).await;
                                    }
                                }

                                cache.start_state_pending_txn().await;
                                let txns = cache.get_all_pending_txn().await;

                                for (_, txn) in txns {
                                    let result = txn.0.check_input(&mut cache).await;
                                    let resource = result.0.extract_resource();

                                    if let Ok(resource) = resource {
                                        resource.0.apply_to_cache(&mut cache).await;
                                    }
                                }

                                poke_the_subs::<Subscribe>(
                                    &mut pool_of_pokers,
                                    &pool_of_subscribes,
                                    &subs_to_poke,
                                )
                                .await;
                            }
                            MessageFromServer::Resources(resources) => {
                                cache.clear_state_pending_txn().await;
                                let mut subs_to_poke = HashSet::new();

                                for resource in resources {
                                    resource.0.apply_to_cache(&mut cache).await;
                                    add_subs(&mut subs_to_poke, resource.0.subs_to_poke());
                                }

                                poke_the_subs::<Subscribe>(
                                    &mut pool_of_pokers,
                                    &pool_of_subscribes,
                                    &subs_to_poke,
                                )
                                .await;

                                cache.start_state_pending_txn().await;
                                let txns = cache.get_all_pending_txn().await;

                                for (_, txn) in txns {
                                    let result = txn.0.check_input(&mut cache).await;
                                    let resource = result.0.extract_resource();

                                    if let Ok(resource) = resource {
                                        resource.0.apply_to_cache(&mut cache).await;
                                    }
                                }
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
                                let result = data.0.check_input(&mut cache).await;
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
                                let result = data.0.check_input(&mut cache).await;

                                let _ = sender
                                    .send(Response::Data {
                                        is_response_from_server: false,
                                        data:                    result,
                                    })
                                    .await;

                                if Aio::is_online(&is_online).await {
                                    let data =
                                        cache.encode_the_inputs(vec![(txn_number, data)]).await;

                                    Aio::send_to_network(&mut sender_to_network, data).await;

                                    pool_of_senders.insert(txn_number, sender);
                                } else {
                                    let _ = sender.send(Response::ServerCannotBeReached).await;
                                    let _ = sender.send(Response::CloseTheChannel).await;
                                };
                            }
                            CachingStrategy::ReadServerFirst => todo!(),
                            CachingStrategy::ReadServerOnly => {
                                if Aio::is_online(&is_online).await {
                                    let data =
                                        cache.encode_the_inputs(vec![(txn_number, data)]).await;

                                    Aio::send_to_network(&mut sender_to_network, data).await;

                                    pool_of_senders.insert(txn_number, sender);
                                } else {
                                    let _ = sender.send(Response::ServerCannotBeReached).await;
                                    let _ = sender.send(Response::CloseTheChannel).await;
                                };
                            }
                            CachingStrategy::WriteCacheOnly => {
                                let result = data.0.check_input(&mut cache).await;

                                let mut subs_to_poke = HashSet::new();
                                add_subs(&mut subs_to_poke, result.0.subs_to_poke());

                                let resource = result.0.extract_resource();

                                if let Ok(resource) = resource {
                                    resource.0.apply_to_cache(&mut cache).await;
                                }
                                cache.write_input_to_cache(txn_number, data).await;

                                poke_the_subs::<Subscribe>(
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
                                let result = data.0.check_input(&mut cache).await;

                                let mut subs_to_poke = HashSet::new();
                                add_subs(&mut subs_to_poke, result.0.subs_to_poke());

                                let resource = result.0.extract_resource();

                                if let Ok(resource) = resource {
                                    resource.0.apply_to_cache(&mut cache).await;
                                }
                                cache.write_input_to_cache(txn_number, data.clone()).await;

                                poke_the_subs::<Subscribe>(
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

                                if Aio::is_online(&is_online).await {
                                    let data =
                                        cache.encode_the_inputs(vec![(txn_number, data)]).await;

                                    Aio::send_to_network(&mut sender_to_network, data).await;

                                    pool_of_senders.insert(txn_number, sender);
                                } else {
                                    let _ = sender.send(Response::ServerCannotBeReached).await;
                                    let _ = sender.send(Response::CloseTheChannel).await;
                                };
                            }
                            CachingStrategy::WriteServerFirst => todo!(),
                            CachingStrategy::WriteServerOnly => {
                                if Aio::is_online(&is_online).await {
                                    let data =
                                        cache.encode_the_inputs(vec![(txn_number, data)]).await;

                                    Aio::send_to_network(&mut sender_to_network, data).await;

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

async fn poke_the_subs<Subscribe: 'static + Hash + Eq>(
    pool_of_pokers: &mut HashMap<u16, MpscSender<()>>,
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

fn add_subs(subs_to_poke: &mut HashSet<Subscribe>, subs: &[Subscribe]) {
    for sub in subs {
        subs_to_poke.insert(*sub);
    }
}
