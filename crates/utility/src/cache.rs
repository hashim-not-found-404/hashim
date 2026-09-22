use crate::dtos::TraitOperationDTOError;
use crate::dtos::TraitOperationDTOInput;
use crate::dtos::TraitOperationDTOOk;
use crate::dtos::Txn;
use crate::dtos::TxnNumber;
use crate::types::ReadAndSet;
use anyhow::Context;
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
use std::any::Any;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Subscribe(u32);

pub trait MarkerCache: 'static {}

pub trait CacheUtility: Sized + 'static {
    type Cache: MarkerCache;

    fn new() -> impl Future<Output = Result<Self>>;
    fn get_inner_cache(&mut self) -> &mut Self::Cache;

    fn get_all_pending_txn(
        &mut self,
    ) -> impl Future<Output = Result<Vec<Txn<TypeOperationClientInput>>>>;
    fn clear_pending_txn_state(&mut self) -> impl Future<Output = Result<()>>;
    fn start_pending_txn_state(&mut self) -> impl Future<Output = Result<()>>;

    fn delete_input_txn(&mut self, txn_number: TxnNumber) -> impl Future<Output = Result<()>>;
    fn mark_input_txn_as_faild(
        &mut self,
        txn_number: TxnNumber,
    ) -> impl Future<Output = Result<()>>;

    fn write_input_to_cache(
        &mut self,
        txn_number: TxnNumber,
        input: TypeOperationClientInput,
    ) -> impl Future<Output = Result<()>>;

    fn write_error_to_cache(
        &mut self,
        txn_number: TxnNumber,
        error: TypeOperationClientError,
    ) -> impl Future<Output = Result<()>>;

    fn encode_the_inputs(
        &mut self,
        inputs: Vec<Txn<TypeOperationClientInput>>,
    ) -> impl Future<Output = Result<Vec<u8>>>;

    fn decode_the_response(resp: Vec<u8>) -> Result<MessageFromServer>;
}

pub trait TraitOperationClientInput: TraitOperationDTOInput + DynClone {
    fn user_uuid(&self) -> Option<[u8; 16]>;
}

pub trait TraitOperationClientOk: TraitOperationDTOOk {
    fn subs_to_poke(&self) -> &'static [Subscribe];
}

pub trait TraitOperationClientError: TraitOperationDTOError + DynClone {
    fn subs_to_poke(&self) -> &'static [Subscribe];
}

pub trait TraitOperationCacheInput: Any + Debug + DynClone {
    type Cache: MarkerCache;
    fn check_input<'a>(
        &'a self,
        cache: &'a mut Self::Cache,
    ) -> Pin<Box<dyn Future<Output = Result<TypeOperationClientResult>> + 'a>>;
}

pub trait TraitOperationCacheOk: Any + Debug {
    type Cache: MarkerCache;
    fn apply_to_cache<'a>(
        &'a self,
        cache: &'a mut Self::Cache,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>>;
}

pub trait CastClientToCache {
    type Cache: MarkerCache;
    fn cast_input(
        v: TypeOperationClientInput,
    ) -> Result<Box<dyn TraitOperationCacheInput<Cache = Self::Cache>>>;
    fn cast_ok(
        v: TypeOperationClientOk,
    ) -> Result<Box<dyn TraitOperationCacheOk<Cache = Self::Cache>>>;
}

pub type TypeOperationClientInput = Arc<dyn TraitOperationClientInput>;
pub type TypeOperationClientOk = Arc<dyn TraitOperationClientOk>;
pub type TypeOperationClientError = Box<dyn TraitOperationClientError>;
pub type TypeOperationClientResult = Result<TypeOperationClientOk, TypeOperationClientError>;

impl Clone for TypeOperationClientError {
    fn clone(&self) -> Self {
        let clone_box = dyn_clone::clone_box(&**self);
        clone_box
    }
}

pub enum MessageFromServer {
    Error(anyhow::Error),
    Response(Vec<Txn<TypeOperationClientResult>>),
    Resources(Vec<TypeOperationClientOk>),
}

#[derive(Debug)]
pub enum Response {
    CloseTheChannel,
    ServerCannotBeReached,
    Data {
        is_response_from_server: bool,
        data: TypeOperationClientResult,
    },
}

pub enum MessageToCache {
    WeAreBackOnline,
    DataFromServer(Vec<u8>),
    Subscribe {
        component_id: u16,
        list_of_subscribtion: &'static [Subscribe],
        sender: MpscSender<()>,
    },
    UnSubscribe {
        component_id: u16,
    },
    Query {
        strategy: CachingStrategy,
        sender: MpscSender<Response>,
        txn_number: TxnNumber,
        data: TypeOperationClientInput,
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

pub struct CacheStruct {
    sender: MpscSender<MessageToCache>,
}

impl Clone for CacheStruct {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl CacheStruct {
    pub fn new<
        Ch: MarkerCache,
        Cu: CacheUtility<Cache = Ch>,
        CasCh: CastClientToCache<Cache = Ch>,
    >(
        receiver_to_cache: MpscReceiver<MessageToCache>,
        sender_to_cache: MpscSender<MessageToCache>,
        sender_to_network: MpscSender<Vec<u8>>,
        sender_to_error: MpscSender<anyhow::Error>,
        is_online: Arc<RwLock<bool>>,
    ) -> Self {
        Self::cache_actor::<Ch, Cu, CasCh>(
            receiver_to_cache,
            sender_to_network,
            sender_to_error,
            is_online,
        );

        Self {
            sender: sender_to_cache,
        }
    }

    pub async fn send_to_cache_actor(
        &mut self,
        strategy: CachingStrategy,
        txn_number: TxnNumber,
        data: TypeOperationClientInput,
    ) -> Result<MpscReceiver<Response>> {
        let (sender, receiver) = Mpsc::channel();

        self.sender
            .send(MessageToCache::Query {
                strategy,
                sender,
                txn_number,
                data,
            })
            .await?;

        Ok(receiver)
    }

    pub async fn send_subs_to_cache_actor(
        &mut self,
        component_id: u16,
        list_of_subscribtion: &'static [Subscribe],
    ) -> Result<MpscReceiver<()>> {
        let (sender, receiver) = Mpsc::channel();

        self.sender
            .send(MessageToCache::Subscribe {
                component_id,
                list_of_subscribtion,
                sender,
            })
            .await?;

        Ok(receiver)
    }

    pub async fn send_unsubs_to_cache_actor(&mut self, component_id: u16) -> Result<()> {
        self.sender
            .send(MessageToCache::UnSubscribe { component_id })
            .await?;

        Ok(())
    }

    fn cache_actor<
        Ch: MarkerCache,
        Cu: CacheUtility<Cache = Ch>,
        CasCh: CastClientToCache<Cache = Ch>,
    >(
        mut receiver_to_cache: MpscReceiver<MessageToCache>,
        mut sender_to_network: MpscSender<Vec<u8>>,
        mut sender_to_error: MpscSender<anyhow::Error>,
        is_online: Arc<RwLock<bool>>,
    ) {
        Rt::spawn_local(async move {
            let mut pool_of_senders =
                HashMap::<TxnNumber, MpscSender<Response>>::with_capacity(100);
            let mut pool_of_pokers = HashMap::<u16, MpscSender<()>>::with_capacity(10);
            let mut pool_of_subscribes = HashMap::<Subscribe, HashSet<u16>>::with_capacity(100);

            let mut cache = Cu::new().await.unwrap();

            loop {
                match receiver_to_cache.recv().await.unwrap() {
                    MessageToCache::WeAreBackOnline => {
                        let txns = cache.get_all_pending_txn().await.unwrap();
                        if txns.is_empty() {
                            continue;
                        }
                        let txns = cache.encode_the_inputs(txns).await.unwrap();
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

                                cache.clear_pending_txn_state().await.unwrap();

                                for Txn {
                                    txn_number,
                                    operation,
                                } in response
                                {
                                    cache.delete_input_txn(txn_number).await.unwrap();

                                    match &operation {
                                        Ok(ok) => {
                                            add_subs(&mut subs_to_poke, ok.subs_to_poke());
                                            let ok = CasCh::cast_ok(ok.clone()).unwrap();
                                            ok.apply_to_cache(cache.get_inner_cache())
                                                .await
                                                .unwrap();
                                            cache.delete_input_txn(txn_number).await.unwrap();
                                        }
                                        Err(err) => {
                                            add_subs(&mut subs_to_poke, err.subs_to_poke());
                                            cache
                                                .mark_input_txn_as_faild(txn_number)
                                                .await
                                                .unwrap();
                                            cache
                                                .write_error_to_cache(txn_number, err.clone())
                                                .await
                                                .unwrap();
                                        }
                                    }

                                    let sender = pool_of_senders.remove(&txn_number);
                                    if let Some(mut sender) = sender {
                                        let _ = sender
                                            .send(Response::Data {
                                                is_response_from_server: true,
                                                data: operation,
                                            })
                                            .await
                                            .unwrap();
                                        let _ =
                                            sender.send(Response::CloseTheChannel).await.unwrap();
                                    }
                                }

                                cache.start_pending_txn_state().await.unwrap();
                                let txns = cache.get_all_pending_txn().await.unwrap();

                                for txn in txns {
                                    let input = CasCh::cast_input(txn.operation).unwrap();
                                    let result =
                                        input.check_input(cache.get_inner_cache()).await.unwrap();

                                    if let Ok(resource) = result {
                                        let ok = CasCh::cast_ok(resource.clone()).unwrap();
                                        ok.apply_to_cache(cache.get_inner_cache()).await.unwrap();
                                    }
                                }

                                poke_the_subs::<Subscribe>(
                                    &mut pool_of_pokers,
                                    &pool_of_subscribes,
                                    &subs_to_poke,
                                )
                                .await
                                .unwrap();
                            }
                            MessageFromServer::Resources(resources) => {
                                cache.clear_pending_txn_state().await.unwrap();
                                let mut subs_to_poke = HashSet::new();

                                for resource in resources {
                                    let ok = CasCh::cast_ok(resource.clone()).unwrap();
                                    ok.apply_to_cache(cache.get_inner_cache()).await.unwrap();
                                    add_subs(&mut subs_to_poke, resource.subs_to_poke());
                                }

                                poke_the_subs::<Subscribe>(
                                    &mut pool_of_pokers,
                                    &pool_of_subscribes,
                                    &subs_to_poke,
                                )
                                .await
                                .unwrap();

                                cache.start_pending_txn_state().await.unwrap();
                                let txns = cache.get_all_pending_txn().await.unwrap();

                                for Txn { operation, .. } in txns {
                                    let input = CasCh::cast_input(operation).unwrap();
                                    let result =
                                        input.check_input(cache.get_inner_cache()).await.unwrap();

                                    if let Ok(resource) = result {
                                        let ok = CasCh::cast_ok(resource.clone()).unwrap();
                                        ok.apply_to_cache(cache.get_inner_cache()).await.unwrap();
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
                    MessageToCache::UnSubscribe { component_id } => {
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
                    } => match strategy {
                        CachingStrategy::ReadCacheOnly => {
                            let input = CasCh::cast_input(data).unwrap();
                            let result = input.check_input(cache.get_inner_cache()).await.unwrap();

                            let _ = sender
                                .send(Response::Data {
                                    is_response_from_server: false,
                                    data: result,
                                })
                                .await
                                .unwrap();
                            let _ = sender.send(Response::CloseTheChannel).await.unwrap();
                        }
                        CachingStrategy::ReadCacheFirst => todo!(),
                        CachingStrategy::ReadCacheAndServer => {
                            let input = CasCh::cast_input(data.clone()).unwrap();
                            let result = input.check_input(cache.get_inner_cache()).await.unwrap();

                            let _ = sender
                                .send(Response::Data {
                                    is_response_from_server: false,
                                    data: result,
                                })
                                .await
                                .unwrap();

                            if is_online.read() {
                                let data = cache
                                    .encode_the_inputs(vec![Txn {
                                        txn_number,
                                        operation: data,
                                    }])
                                    .await
                                    .unwrap();

                                sender_to_network.send(data).await.unwrap();

                                pool_of_senders.insert(txn_number, sender);
                            } else {
                                let _ = sender.send(Response::ServerCannotBeReached).await.unwrap();
                                let _ = sender.send(Response::CloseTheChannel).await.unwrap();
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
                                    .await
                                    .unwrap();

                                sender_to_network.send(data).await.unwrap();

                                pool_of_senders.insert(txn_number, sender);
                            } else {
                                let _ = sender.send(Response::ServerCannotBeReached).await.unwrap();
                                let _ = sender.send(Response::CloseTheChannel).await.unwrap();
                            }
                        }
                        CachingStrategy::WriteCacheOnly => {
                            let input = CasCh::cast_input(data.clone()).unwrap();
                            let result = input.check_input(cache.get_inner_cache()).await.unwrap();

                            let mut subs_to_poke = HashSet::new();

                            match &result {
                                Ok(ok) => {
                                    add_subs(&mut subs_to_poke, ok.subs_to_poke());
                                    let ok = CasCh::cast_ok(ok.clone()).unwrap();
                                    ok.apply_to_cache(cache.get_inner_cache()).await.unwrap();
                                }
                                Err(err) => {
                                    add_subs(&mut subs_to_poke, err.subs_to_poke());
                                }
                            }
                            cache
                                .write_input_to_cache(txn_number, data.clone())
                                .await
                                .unwrap();

                            poke_the_subs::<Subscribe>(
                                &mut pool_of_pokers,
                                &pool_of_subscribes,
                                &subs_to_poke,
                            )
                            .await
                            .unwrap();

                            let _ = sender
                                .send(Response::Data {
                                    is_response_from_server: false,
                                    data: result,
                                })
                                .await
                                .unwrap();

                            let _ = sender.send(Response::CloseTheChannel).await.unwrap();
                        }
                        CachingStrategy::WriteCacheFirst => todo!(),
                        CachingStrategy::WriteCacheAndServer => {
                            let input = CasCh::cast_input(data.clone()).unwrap();
                            let result = input.check_input(cache.get_inner_cache()).await.unwrap();

                            let mut subs_to_poke = HashSet::new();

                            match &result {
                                Ok(ok) => {
                                    add_subs(&mut subs_to_poke, ok.subs_to_poke());
                                    let ok = CasCh::cast_ok(ok.clone()).unwrap();
                                    ok.apply_to_cache(cache.get_inner_cache()).await.unwrap();
                                }
                                Err(err) => {
                                    add_subs(&mut subs_to_poke, err.subs_to_poke());
                                }
                            }
                            cache
                                .write_input_to_cache(txn_number, data.clone())
                                .await
                                .unwrap();

                            poke_the_subs::<Subscribe>(
                                &mut pool_of_pokers,
                                &pool_of_subscribes,
                                &subs_to_poke,
                            )
                            .await
                            .unwrap();

                            let _ = sender
                                .send(Response::Data {
                                    is_response_from_server: false,
                                    data: result,
                                })
                                .await
                                .unwrap();

                            if is_online.read() {
                                let data = cache
                                    .encode_the_inputs(vec![Txn {
                                        txn_number,
                                        operation: data,
                                    }])
                                    .await
                                    .unwrap();

                                sender_to_network.send(data).await.unwrap();

                                pool_of_senders.insert(txn_number, sender);
                            } else {
                                let _ = sender.send(Response::ServerCannotBeReached).await.unwrap();
                                let _ = sender.send(Response::CloseTheChannel).await.unwrap();
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
                                    .await
                                    .unwrap();

                                sender_to_network.send(data).await.unwrap();

                                pool_of_senders.insert(txn_number, sender);
                            } else {
                                let _ = sender.send(Response::ServerCannotBeReached).await.unwrap();
                                let _ = sender.send(Response::CloseTheChannel).await.unwrap();
                            }
                        }
                    },
                }
            }
        });
    }
}

async fn poke_the_subs<Subscribe: 'static + Hash + Eq>(
    pool_of_pokers: &mut HashMap<u16, MpscSender<()>>,
    pool_of_subscribes: &HashMap<Subscribe, HashSet<u16>>,
    subs_to_poke: &HashSet<Subscribe>,
) -> Result<()> {
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
        let sender = pool_of_pokers.get_mut(i).context("missing value")?;
        let _ = sender.send(()).await;
    }

    Ok(())
}

fn add_subs(subs_to_poke: &mut HashSet<Subscribe>, subs: &[Subscribe]) {
    for sub in subs {
        subs_to_poke.insert(*sub);
    }
}
