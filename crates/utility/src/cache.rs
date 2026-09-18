use crate::dtos::OperationsError;
use crate::dtos::OperationsInput;
use crate::dtos::OperationsOk;
use crate::dtos::Txn;
use crate::dtos::TxnNumber;
use crate::types::ReadAndSet;
use anyhow::Result;
use dyn_clone::DynClone;
use infrastructure::actors::Mpsc;
use infrastructure::actors::MpscReceiver;
use infrastructure::actors::MpscSender;
use infrastructure::actors::MultiProducerSingleConsumer;
use infrastructure::actors::Receiver;
use infrastructure::actors::Sender;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Subscribe(u32);

pub trait CacheUtility: Sized + 'static {
    type Cache;

    fn new() -> impl Future<Output = Self>;

    fn get_inner_cache(&mut self) -> &mut Self::Cache;

    fn get_all_pending_txn(&mut self) -> impl Future<Output = Vec<Txn<OpInput<Self::Cache>>>>;
    fn clear_pending_txn_state(&mut self) -> impl Future<Output = ()>;
    fn start_pending_txn_state(&mut self) -> impl Future<Output = ()>;

    fn delete_input_txn(&mut self, txn_number: TxnNumber) -> impl Future<Output = ()>;
    fn mark_input_txn_as_faild(&mut self, txn_number: TxnNumber) -> impl Future<Output = ()>;

    fn write_input_to_cache(
        &mut self,
        txn_number: TxnNumber,
        input: OpInput<Self::Cache>,
    ) -> impl Future<Output = ()>;

    fn write_error_to_cache(
        &mut self,
        txn_number: TxnNumber,
        error: OpError,
    ) -> impl Future<Output = ()>;

    fn encode_the_inputs(
        &mut self,
        inputs: Vec<Txn<OpInput<Self::Cache>>>,
    ) -> impl Future<Output = Vec<u8>>;

    fn decode_the_response(resp: Vec<u8>) -> Result<MessageFromServer<Self::Cache>>;
}

pub trait OpInputTrait<Ch>: Debug + DynClone {
    fn into_serde(self: Box<Self>) -> Box<dyn OperationsInput>;
    fn check_input<'a>(
        &'a self,
        cache: &'a mut Ch,
    ) -> Pin<Box<dyn Future<Output = OpResult<Ch>> + 'a>>;
    fn user_uuid(&self) -> Option<[u8; 16]>;
}

pub trait OpOkTrait<Ch>: Debug {
    fn into_serde(self: Box<Self>) -> Box<dyn OperationsOk>;
    fn apply_to_cache(&self, cache: &mut Ch) -> Pin<Box<dyn Future<Output = ()>>>;
    fn subs_to_poke(&self) -> &'static [Subscribe];
}

pub trait OpErrorTrait: Debug + DynClone {
    fn into_serde(self: Box<Self>) -> Box<dyn OperationsError>;
    fn subs_to_poke(&self) -> &'static [Subscribe];
}

#[derive(Debug)]
pub struct OpInput<Ch>(pub Arc<dyn OpInputTrait<Ch>>);
#[derive(Debug)]
pub struct OpOk<Ch>(pub Box<dyn OpOkTrait<Ch>>);
#[derive(Debug)]
pub struct OpError(pub Box<dyn OpErrorTrait>);
pub type OpResult<Ch> = Result<OpOk<Ch>, OpError>;

// impl<Ch, T: OpInputTrait<Ch> + 'static> From<T> for OpInput<Ch> {
//     fn from(value: T) -> Self {
//         Self(Arc::new(value))
//     }
// }

impl<Ch> Clone for OpInput<Ch> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Clone for OpError {
    fn clone(&self) -> Self {
        let clone_box = dyn_clone::clone_box(&*self.0);
        Self(clone_box)
    }
}

pub enum MessageFromServer<Ch> {
    Error(anyhow::Error),
    Response(Vec<Txn<OpResult<Ch>>>),
    Resources(Vec<OpOk<Ch>>),
}

#[derive(Debug)]
pub enum Response<Ch> {
    CloseTheChannel,
    ServerCannotBeReached,
    Data {
        is_response_from_server: bool,
        data:                    OpResult<Ch>,
    },
}

pub enum MessageToCache<Ch> {
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

pub struct CacheStruct<Ch> {
    sender: MpscSender<MessageToCache<Ch>>,
}

impl<Ch> Clone for CacheStruct<Ch> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<Ch: 'static> CacheStruct<Ch> {
    pub fn new<Cu: CacheUtility<Cache = Ch>>(
        receiver_to_cache: MpscReceiver<MessageToCache<Ch>>,
        sender_to_cache: MpscSender<MessageToCache<Ch>>,
        sender_to_network: MpscSender<Vec<u8>>,
        sender_to_error: MpscSender<anyhow::Error>,
        is_online: Arc<RwLock<bool>>,
    ) -> Self {
        Self::cache_actor::<Cu>(receiver_to_cache, sender_to_network, sender_to_error, is_online);

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

    fn cache_actor<Cu: CacheUtility<Cache = Ch>>(
        mut receiver_to_cache: MpscReceiver<MessageToCache<Ch>>,
        mut sender_to_network: MpscSender<Vec<u8>>,
        mut sender_to_error: MpscSender<anyhow::Error>,
        is_online: Arc<RwLock<bool>>,
    ) {
        Rt::spawn_local(async move {
            let mut pool_of_senders =
                HashMap::<TxnNumber, MpscSender<Response<Ch>>>::with_capacity(100);
            let mut pool_of_pokers = HashMap::<u16, MpscSender<()>>::with_capacity(10);
            let mut pool_of_subscribes = HashMap::<Subscribe, HashSet<u16>>::with_capacity(100);

            let mut cache = Cu::new().await;

            loop {
                match receiver_to_cache.recv().await.unwrap() {
                    MessageToCache::WeAreBackOnline => {
                        let txns = cache.get_all_pending_txn().await;
                        if txns.is_empty() {
                            continue;
                        }
                        let txns = cache.encode_the_inputs(txns).await;
                        sender_to_network.send(txns).await.unwrap();
                    }
                    MessageToCache::DataFromServer(raw_data) => {
                        let message_type = match Cu::decode_the_response(raw_data) {
                            Ok(ok) => ok,
                            Err(err) => {
                                sender_to_error.send(err).await.unwrap();
                                continue;
                            }
                        };

                        match message_type {
                            MessageFromServer::Error(err) => {
                                sender_to_error.send(err).await.unwrap();
                            }
                            MessageFromServer::Response(response) => {
                                let mut subs_to_poke = HashSet::new();

                                cache.clear_pending_txn_state().await;

                                for Txn {
                                    txn_number,
                                    operation,
                                } in response
                                {
                                    cache.delete_input_txn(txn_number).await;

                                    match &operation {
                                        Ok(ok) => {
                                            add_subs(&mut subs_to_poke, ok.0.subs_to_poke());
                                            ok.0.apply_to_cache(cache.get_inner_cache()).await;
                                            cache.delete_input_txn(txn_number).await;
                                        }
                                        Err(err) => {
                                            add_subs(&mut subs_to_poke, err.0.subs_to_poke());
                                            cache.mark_input_txn_as_faild(txn_number).await;
                                            cache
                                                .write_error_to_cache(txn_number, err.clone())
                                                .await;
                                        }
                                    }

                                    let sender = pool_of_senders.remove(&txn_number);
                                    if let Some(mut sender) = sender {
                                        let _ = sender
                                            .send(Response::Data {
                                                is_response_from_server: true,
                                                data:                    operation,
                                            })
                                            .await;
                                        let _ = sender.send(Response::CloseTheChannel).await;
                                    }
                                }

                                cache.start_pending_txn_state().await;
                                let txns = cache.get_all_pending_txn().await;

                                for txn in txns {
                                    let result =
                                        txn.operation.0.check_input(cache.get_inner_cache()).await;

                                    if let Ok(resource) = result {
                                        resource.0.apply_to_cache(cache.get_inner_cache()).await;
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
                                cache.clear_pending_txn_state().await;
                                let mut subs_to_poke = HashSet::new();

                                for resource in resources {
                                    resource.0.apply_to_cache(cache.get_inner_cache()).await;
                                    add_subs(&mut subs_to_poke, resource.0.subs_to_poke());
                                }

                                poke_the_subs::<Subscribe>(
                                    &mut pool_of_pokers,
                                    &pool_of_subscribes,
                                    &subs_to_poke,
                                )
                                .await;

                                cache.start_pending_txn_state().await;
                                let txns = cache.get_all_pending_txn().await;

                                for Txn {
                                    operation,
                                    ..
                                } in txns
                                {
                                    let result =
                                        operation.0.check_input(cache.get_inner_cache()).await;

                                    if let Ok(resource) = result {
                                        resource.0.apply_to_cache(cache.get_inner_cache()).await;
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
                                let result = data.0.check_input(cache.get_inner_cache()).await;
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
                                let result = data.0.check_input(cache.get_inner_cache()).await;

                                let _ = sender
                                    .send(Response::Data {
                                        is_response_from_server: false,
                                        data:                    result,
                                    })
                                    .await;

                                if is_online.read() {
                                    let data = cache
                                        .encode_the_inputs(vec![Txn {
                                            txn_number,
                                            operation: data,
                                        }])
                                        .await;

                                    sender_to_network.send(data).await.unwrap();

                                    pool_of_senders.insert(txn_number, sender);
                                } else {
                                    let _ = sender.send(Response::ServerCannotBeReached).await;
                                    let _ = sender.send(Response::CloseTheChannel).await;
                                }
                            }
                            CachingStrategy::ReadServerFirst => todo!(),
                            CachingStrategy::ReadServerOnly => {
                                if is_online.read() {
                                    let data = cache
                                        .encode_the_inputs(vec![Txn {
                                            txn_number,
                                            operation: data,
                                        }])
                                        .await;

                                    sender_to_network.send(data).await.unwrap();

                                    pool_of_senders.insert(txn_number, sender);
                                } else {
                                    let _ = sender.send(Response::ServerCannotBeReached).await;
                                    let _ = sender.send(Response::CloseTheChannel).await;
                                }
                            }
                            CachingStrategy::WriteCacheOnly => {
                                let result = data.0.check_input(cache.get_inner_cache()).await;

                                let mut subs_to_poke = HashSet::new();

                                match &result {
                                    Ok(ok) => {
                                        add_subs(&mut subs_to_poke, ok.0.subs_to_poke());
                                        ok.0.apply_to_cache(cache.get_inner_cache()).await;
                                    }
                                    Err(err) => {
                                        add_subs(&mut subs_to_poke, err.0.subs_to_poke());
                                    }
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

                                let _ = sender.send(Response::CloseTheChannel).await;
                            }
                            CachingStrategy::WriteCacheFirst => todo!(),
                            CachingStrategy::WriteCacheAndServer => {
                                let result = data.0.check_input(cache.get_inner_cache()).await;

                                let mut subs_to_poke = HashSet::new();

                                match &result {
                                    Ok(ok) => {
                                        add_subs(&mut subs_to_poke, ok.0.subs_to_poke());
                                        ok.0.apply_to_cache(cache.get_inner_cache()).await;
                                    }
                                    Err(err) => {
                                        add_subs(&mut subs_to_poke, err.0.subs_to_poke());
                                    }
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

                                if is_online.read() {
                                    let data = cache
                                        .encode_the_inputs(vec![Txn {
                                            txn_number,
                                            operation: data,
                                        }])
                                        .await;

                                    sender_to_network.send(data).await.unwrap();

                                    pool_of_senders.insert(txn_number, sender);
                                } else {
                                    let _ = sender.send(Response::ServerCannotBeReached).await;
                                    let _ = sender.send(Response::CloseTheChannel).await;
                                }
                            }
                            CachingStrategy::WriteServerFirst => todo!(),
                            CachingStrategy::WriteServerOnly => {
                                if is_online.read() {
                                    let data = cache
                                        .encode_the_inputs(vec![Txn {
                                            txn_number,
                                            operation: data,
                                        }])
                                        .await;

                                    sender_to_network.send(data).await.unwrap();

                                    pool_of_senders.insert(txn_number, sender);
                                } else {
                                    let _ = sender.send(Response::ServerCannotBeReached).await;
                                    let _ = sender.send(Response::CloseTheChannel).await;
                                }
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
