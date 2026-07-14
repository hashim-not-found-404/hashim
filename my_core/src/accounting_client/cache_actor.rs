use crate::{
    accounting_client::{cache, network_actor},
    accounting_domain::{cases, request_response, types},
    utility::{
        traits::{self, Receiver, Sender},
        utils::ReadAndSet,
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

#[derive(Clone)]
pub(crate) struct Data {
    pub(crate) is_response_from_server: bool,
    pub(crate) data: request_response::push_data::OperationsResult,
}

#[derive(Clone)]
pub(crate) enum Response {
    CloseTheChannel,
    ServerCannotBeReached,
    Data(Data),
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

pub(crate) enum MessageToCache<Mpsc: traits::MultiProducerSingleConsumer> {
    WeAreBackOnline,
    DataFromServer(Vec<u8>),
    Subscribe {
        component_id: u16,
        list_of_subscribtion: &'static [types::Subscribe],
        sender: Mpsc::Sender<()>,
    },
    UnSubscribe {
        component_id: u16,
    },
    Query {
        strategy: CachingStrategy,
        sender: Mpsc::Sender<Response>,
        data: request_response::push_data::OperationsInput,
    },
}

pub(crate) struct CacheStruct<Mpsc>
where
    Mpsc: traits::MultiProducerSingleConsumer,
{
    sender: Mpsc::Sender<MessageToCache<Mpsc>>,
}

impl<Mpsc: traits::MultiProducerSingleConsumer> Clone for CacheStruct<Mpsc> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<Mpsc> CacheStruct<Mpsc>
where
    Mpsc: traits::MultiProducerSingleConsumer,
{
    pub(crate) fn new<
        Rt: traits::Runtime,
        Id: cases::RowId,
        Ch: cache::Cache,
        Ed: traits::Coding,
        Rn: traits::RandomNumber,
    >(
        receiver_to_cache: Mpsc::Receiver<MessageToCache<Mpsc>>,
        sender_to_cache: Mpsc::Sender<MessageToCache<Mpsc>>,
        sender_to_network: Mpsc::Sender<network_actor::MessageToNetwork>,
        sender_to_error: Mpsc::Sender<types::HashimError>,
        is_online: Arc<RwLock<bool>>,
    ) -> Self {
        Self::cache_actor::<Rt, Id, Ch, Ed, Rn>(
            receiver_to_cache,
            sender_to_network.clone(),
            sender_to_error.clone(),
            is_online.clone(),
        );

        Self {
            sender: sender_to_cache,
        }
    }

    pub(crate) async fn send_to_cache_actor(
        &mut self,
        strategy: CachingStrategy,
        data: request_response::push_data::OperationsInput,
    ) -> Mpsc::Receiver<Response> {
        let (sender, receiver) = Mpsc::channel();

        self.sender
            .send(MessageToCache::Query {
                strategy,
                sender,
                data,
            })
            .await
            .unwrap();

        receiver
    }

    pub(crate) async fn send_subs_to_cache_actor(
        &mut self,
        component_id: u16,
        list_of_subscribtion: &'static [types::Subscribe],
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
            .send(MessageToCache::UnSubscribe { component_id })
            .await
            .unwrap();
    }

    fn cache_actor<
        Rt: traits::Runtime,
        Id: cases::RowId,
        Ch: cache::Cache,
        Ed: traits::Coding,
        Rn: traits::RandomNumber,
    >(
        mut receiver_to_cache: Mpsc::Receiver<MessageToCache<Mpsc>>,
        mut sender_to_network: Mpsc::Sender<network_actor::MessageToNetwork>,
        mut sender_to_error: Mpsc::Sender<types::HashimError>,
        is_online: Arc<RwLock<bool>>,
    ) {
        Rt::spawn_local(async move {
            let mut pool_of_senders = HashMap::<u64, Mpsc::Sender<Response>>::with_capacity(100);
            let mut pool_of_pokers = HashMap::<u16, Mpsc::Sender<()>>::with_capacity(10);
            let mut pool_of_subscribes =
                HashMap::<types::Subscribe, HashSet<u16>>::with_capacity(100);

            let mut state = cache::State::<Ch>::new::<Id>().await;

            loop {
                match receiver_to_cache.recv().await.unwrap() {
                    MessageToCache::WeAreBackOnline => {
                        let operations = state.cache.get_all_txn_input().await;

                        Self::prepare_txn_and_send_to_network::<Ch, Id, Ed>(
                            &mut sender_to_network,
                            operations,
                            &state,
                        )
                        .await;
                    }
                    MessageToCache::DataFromServer(raw_data) => {
                        let message_type =
                            match Ed::decode::<request_response::messages::FromServer>(&raw_data) {
                                Ok(message_type) => message_type,
                                Err(_) => {
                                    sender_to_error
                                        .send(types::HashimError::InvalidDataFormat.into())
                                        .await
                                        .unwrap();
                                    continue;
                                }
                            };

                        match message_type {
                            request_response::messages::FromServer::Error(err) => {
                                sender_to_error.send(err.into()).await.unwrap()
                            }
                            request_response::messages::FromServer::PushData(results) => {
                                let mut subs_to_poke = HashSet::new();

                                for txn in results.operations {
                                    let data = txn.operation;
                                    let resource = data.extract_resource();
                                    collect_subs_to_poke(&mut subs_to_poke, &resource);
                                    state.cache.write_resource(&resource).await;

                                    let txn = request_response::push_data::Txn {
                                        txn_number: txn.txn_number,
                                        operation: data.clone(),
                                    };

                                    if data.is_ok() {
                                        state.cache.delete_txn_input(&txn.txn_number).await;
                                    } else {
                                        state.cache.mark_txn_input_as_faild(&txn.txn_number).await;
                                        state.cache.write_txn_result(&txn).await;
                                    }

                                    let sender = pool_of_senders.remove(&txn.txn_number);
                                    if let Some(mut sender) = sender {
                                        let _ = sender
                                            .send(Response::Data(Data {
                                                is_response_from_server: true,
                                                data,
                                            }))
                                            .await;
                                        let _ = sender.send(Response::CloseTheChannel).await;
                                    }
                                }

                                state.state_of_pending_txn =
                                    cache::tables::StateOfPendingTxn::default();

                                let txns = state.cache.get_all_txn_input().await;

                                for op in txns {
                                    op.operation
                                        .run_operation_check_apply::<Id, Ch>(
                                            &mut state,
                                            &mut subs_to_poke,
                                        )
                                        .await;
                                }

                                poke_the_subs::<Mpsc>(
                                    &mut pool_of_pokers,
                                    &pool_of_subscribes,
                                    &subs_to_poke,
                                )
                                .await;
                            }
                            request_response::messages::FromServer::Resources(resource) => {
                                state.cache.write_resource(&resource).await;

                                let mut subs_to_poke = HashSet::new();
                                collect_subs_to_poke(&mut subs_to_poke, &resource);
                                poke_the_subs::<Mpsc>(
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
                    MessageToCache::UnSubscribe { component_id } => {
                        pool_of_pokers.remove(&component_id);

                        for (_, components) in pool_of_subscribes.iter_mut() {
                            components.remove(&component_id);
                        }

                        pool_of_subscribes.retain(|_, components| !components.is_empty());
                    }
                    MessageToCache::Query {
                        strategy,
                        mut sender,
                        data,
                    } => match strategy {
                        CachingStrategy::ReadCacheOnly => {
                            let result = data.run_operation_check::<Id, Ch>(&mut state).await;
                            let _ = sender
                                .send(Response::Data(Data {
                                    is_response_from_server: false,
                                    data: result,
                                }))
                                .await;
                            let _ = sender.send(Response::CloseTheChannel).await;
                        }
                        CachingStrategy::ReadCacheFirst => todo!(),
                        CachingStrategy::ReadCacheAndServer => {
                            let txn_number = Rn::generate();

                            let result = data.run_operation_check::<Id, Ch>(&mut state).await;

                            let _ = sender
                                .send(Response::Data(Data {
                                    is_response_from_server: false,
                                    data: result,
                                }))
                                .await;

                            let operations = vec![request_response::push_data::Txn {
                                txn_number,
                                operation: data,
                            }];

                            if is_online.read() {
                                Self::prepare_txn_and_send_to_network::<Ch, Id, Ed>(
                                    &mut sender_to_network,
                                    operations,
                                    &state,
                                )
                                .await;

                                pool_of_senders.insert(txn_number, sender);
                            } else {
                                let _ = sender.send(Response::ServerCannotBeReached).await;
                                let _ = sender.send(Response::CloseTheChannel).await;
                            };
                        }
                        CachingStrategy::ReadServerFirst => todo!(),
                        CachingStrategy::ReadServerOnly => todo!(),
                        CachingStrategy::WriteCacheOnly => todo!(),
                        CachingStrategy::WriteCacheFirst => todo!(),
                        CachingStrategy::WriteCacheAndServer => {
                            let txn_number = Rn::generate();

                            let mut subs_to_poke = HashSet::new();
                            let result = data
                                .run_operation_check_apply_write::<Id, Ch>(
                                    txn_number,
                                    &mut state,
                                    &mut subs_to_poke,
                                )
                                .await;

                            poke_the_subs::<Mpsc>(
                                &mut pool_of_pokers,
                                &pool_of_subscribes,
                                &subs_to_poke,
                            )
                            .await;

                            let _ = sender
                                .send(Response::Data(Data {
                                    is_response_from_server: false,
                                    data: result,
                                }))
                                .await;

                            let operations = vec![request_response::push_data::Txn {
                                txn_number,
                                operation: data,
                            }];

                            if is_online.read() {
                                Self::prepare_txn_and_send_to_network::<Ch, Id, Ed>(
                                    &mut sender_to_network,
                                    operations,
                                    &state,
                                )
                                .await;

                                pool_of_senders.insert(txn_number, sender);
                            } else {
                                let _ = sender.send(Response::ServerCannotBeReached).await;
                                let _ = sender.send(Response::CloseTheChannel).await;
                            };
                        }
                        CachingStrategy::WriteServerFirst => todo!(),
                        CachingStrategy::WriteServerOnly => todo!(),
                    },
                }
            }
        });
    }

    async fn prepare_txn_and_send_to_network<
        Ch: cache::Cache,
        Id: cases::RowId,
        Ed: traits::Coding,
    >(
        sender_to_network: &mut Mpsc::Sender<network_actor::MessageToNetwork>,
        operations: Vec<
            request_response::push_data::Txn<request_response::push_data::OperationsInput>,
        >,
        state: &cache::State<Ch>,
    ) {
        if operations.is_empty() {
            return;
        }

        let mut jwts = Vec::new();
        for operation in &operations {
            if let Some(user_uuid) = operation.operation.get_user_uuid() {
                if let Some(jwt) = state.cache.get_jwt(user_uuid).await {
                    jwts.push(jwt)
                }
            }
        }

        let mut operations1 = Vec::with_capacity(operations.len());

        for i in operations {
            operations1.push(request_response::push_data::Txn {
                txn_number: i.txn_number,
                operation: i.operation,
            });
        }

        let t = request_response::messages::FromClient {
            jwts,
            nonce: Id::generate(),
            operations: operations1,
        };

        let t = Ed::encode(&t);

        sender_to_network.send(t).await.unwrap();
    }
}

pub(crate) fn collect_subs_to_poke(
    subs_to_poke: &mut HashSet<types::Subscribe>,
    resource: &Vec<types::ResourceInfo>,
) {
    for resource in resource {
        match resource.resource {
            types::Resource::Jwt(_) => {}
            types::Resource::TableUserFieldName(_) => {
                subs_to_poke.insert(types::Subscribe::TableUserFieldName);
            }
            types::Resource::TableUserFieldId(_) => {
                subs_to_poke.insert(types::Subscribe::TableUserFieldId);
            }
            types::Resource::TableCompanyFieldName(_) => {
                subs_to_poke.insert(types::Subscribe::TableCompanyFieldName);
            }
            types::Resource::TableCompanyBranchFieldName(_) => {
                subs_to_poke.insert(types::Subscribe::TableCompanyBranchFieldName);
            }
            types::Resource::TableCompanyBranchFieldCompanyBelong(_) => {
                subs_to_poke.insert(types::Subscribe::TableCompanyBranchFieldCompanyBelong);
            }
            types::Resource::TableCompanyBranchFieldCurrency(_) => {
                subs_to_poke.insert(types::Subscribe::TableCompanyBranchFieldCurrency);
            }
            types::Resource::TableCompanyBranchFieldLocation(_) => {
                subs_to_poke.insert(types::Subscribe::TableCompanyBranchFieldLocation);
            }
            types::Resource::TableCompanyFieldCurrency(_) => {
                subs_to_poke.insert(types::Subscribe::TableCompanyFieldCurrency);
            }
            types::Resource::TableAccessControlForCompanyFieldRole(_) => {
                subs_to_poke.insert(types::Subscribe::TableAccessControlForCompanyFieldRole);
            }
            types::Resource::TableAccessControlForCompanyFieldUser(_) => {
                subs_to_poke.insert(types::Subscribe::TableAccessControlForCompanyFieldUser);
            }
            types::Resource::TableAccessControlForCompanyFieldDataGroup(_) => {
                subs_to_poke.insert(types::Subscribe::TableAccessControlForCompanyFieldDataGroup);
            }
            types::Resource::TableAccessControlForCompanyBranchFieldRole(_) => {
                subs_to_poke.insert(types::Subscribe::TableAccessControlForCompanyBranchFieldRole);
            }
            types::Resource::TableAccessControlForCompanyBranchFieldUser(_) => {
                subs_to_poke.insert(types::Subscribe::TableAccessControlForCompanyBranchFieldUser);
            }
            types::Resource::TableAccessControlForCompanyBranchFieldDataGroup(_) => {
                subs_to_poke
                    .insert(types::Subscribe::TableAccessControlForCompanyBranchFieldDataGroup);
            }
        }
    }
}

async fn poke_the_subs<Mpsc: traits::MultiProducerSingleConsumer>(
    pool_of_pokers: &mut HashMap<u16, Mpsc::Sender<()>>,
    pool_of_subscribes: &HashMap<types::Subscribe, HashSet<u16>>,
    subs_to_poke: &HashSet<types::Subscribe>,
) {
    let mut components_to_poke = HashSet::new();

    for one_sub in subs_to_poke {
        let a = match pool_of_subscribes.get(&one_sub) {
            Some(a) => a,
            None => continue,
        };

        for a in a {
            components_to_poke.insert(a.clone());
        }
    }

    for i in components_to_poke {
        let sender = pool_of_pokers.get_mut(&i).unwrap();
        sender.send(()).await.unwrap();
    }
}
