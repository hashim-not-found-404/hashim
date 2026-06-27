use crate::prelude::*;

#[derive(Clone)]
pub struct Data {
    pub is_response_from_server: bool,
    pub data: push_data::OperationsResult,
}

#[derive(Clone)]
pub enum Response {
    CloseTheChannel,
    ServerCannotBeReached,
    Data(Data),
}

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

pub(crate) enum MessageToCache<At: AllClientTypes> {
    WeAreBackOnline,
    DataFromServer(Vec<u8>),
    Subscribe {
        component_id: u16,
        list_of_subscribtion: &'static [server_methods::Subscribe],
        sender: <At::Mpsc as MultiProducerSingleConsumer>::Sender<()>,
    },
    UnSubscribe {
        component_id: u16,
    },
    Query {
        strategy: CachingStrategy,
        sender: <At::Mpsc as MultiProducerSingleConsumer>::Sender<Response>,
        data: push_data::OperationsInput,
    },
}

pub struct CacheStruct<At>
where
    At: AllClientTypes,
{
    sender: <At::Mpsc as MultiProducerSingleConsumer>::Sender<MessageToCache<At>>,
}

impl<At: AllClientTypes> Clone for CacheStruct<At> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<At> CacheStruct<At>
where
    At: AllClientTypes,
{
    pub(crate) fn new(
        receiver_to_cache: <At::Mpsc as MultiProducerSingleConsumer>::Receiver<MessageToCache<At>>,
        sender_to_cache: <At::Mpsc as MultiProducerSingleConsumer>::Sender<MessageToCache<At>>,
        sender_to_network: <At::Mpsc as MultiProducerSingleConsumer>::Sender<
            network_actor::MessageToNetwork,
        >,
        sender_to_error: <At::Mpsc as MultiProducerSingleConsumer>::Sender<HashimError>,
        is_online: Arc<RwLock<bool>>,
    ) -> Self {
        Self::cache_actor(
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
        data: push_data::OperationsInput,
    ) -> <At::Mpsc as MultiProducerSingleConsumer>::Receiver<Response> {
        let (sender, receiver) = At::Mpsc::channel();

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
        list_of_subscribtion: &'static [server_methods::Subscribe],
    ) -> <At::Mpsc as MultiProducerSingleConsumer>::Receiver<()> {
        let (sender, receiver) = At::Mpsc::channel();

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

    fn cache_actor(
        mut receiver_to_cache: <At::Mpsc as MultiProducerSingleConsumer>::Receiver<
            MessageToCache<At>,
        >,
        mut sender_to_network: <At::Mpsc as MultiProducerSingleConsumer>::Sender<
            network_actor::MessageToNetwork,
        >,
        mut sender_to_error: <At::Mpsc as MultiProducerSingleConsumer>::Sender<HashimError>,
        is_online: Arc<RwLock<bool>>,
    ) {
        At::Rt::spawn_local(async move {
            let mut pool_of_senders = HashMap::<
                u64,
                <At::Mpsc as MultiProducerSingleConsumer>::Sender<Response>,
            >::with_capacity(100);
            let mut pool_of_pokers = HashMap::<
                u16,
                <At::Mpsc as MultiProducerSingleConsumer>::Sender<()>,
            >::with_capacity(10);
            let mut pool_of_subscribes =
                HashMap::<server_methods::Subscribe, HashSet<u16>>::with_capacity(100);

            let mut state = cache::State::<At::Ch>::new().await;

            loop {
                match receiver_to_cache.recv().await.unwrap() {
                    MessageToCache::WeAreBackOnline => {
                        let operations = state.cache.get_all_txn_input().await;

                        Self::prepare_txn_and_send_to_network(
                            &mut sender_to_network,
                            operations,
                            &state,
                        )
                        .await;
                    }
                    MessageToCache::DataFromServer(raw_data) => {
                        let message_type = match At::Ed::decode::<messages::FromServer>(&raw_data) {
                            Ok(message_type) => message_type,
                            Err(_) => {
                                sender_to_error
                                    .send(HashimError::InvalidDataFormat.into())
                                    .await
                                    .unwrap();
                                continue;
                            }
                        };

                        match message_type {
                            messages::FromServer::Error(err) => {
                                sender_to_error.send(err.into()).await.unwrap()
                            }
                            messages::FromServer::PushData(results) => {
                                let mut subs_to_poke = HashSet::new();

                                for txn in results.operations {
                                    let data = txn.operation;
                                    let resource = data.extract_resource();
                                    collect_subs_to_poke(&mut subs_to_poke, &resource);
                                    state.cache.write_resource(&resource).await;

                                    let txn = push_data::Txn {
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

                                state.state_of_pending_txn = cache::StateOfPendingTxn::default();

                                let txns = state.cache.get_all_txn_input().await;

                                for op in txns {
                                    op.operation
                                        .run_operation_check_apply(&mut state, &mut subs_to_poke)
                                        .await;
                                }

                                poke_the_subs::<At>(
                                    &mut pool_of_pokers,
                                    &pool_of_subscribes,
                                    &subs_to_poke,
                                )
                                .await;
                            }
                            messages::FromServer::Resources(resource) => {
                                state.cache.write_resource(&resource).await;

                                let mut subs_to_poke = HashSet::new();
                                collect_subs_to_poke(&mut subs_to_poke, &resource);
                                poke_the_subs::<At>(
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
                            let result = data.run_operation_check(&mut state).await;
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
                            let txn_number = At::Rn::generate();

                            let result = data.run_operation_check(&mut state).await;

                            let _ = sender
                                .send(Response::Data(Data {
                                    is_response_from_server: false,
                                    data: result,
                                }))
                                .await;

                            let operations = vec![push_data::Txn {
                                txn_number,
                                operation: data,
                            }];

                            if is_online.read() {
                                Self::prepare_txn_and_send_to_network(
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
                            let txn_number = At::Rn::generate();

                            let mut subs_to_poke = HashSet::new();
                            let result = data
                                .run_operation_check_apply_write(
                                    txn_number,
                                    &mut state,
                                    &mut subs_to_poke,
                                )
                                .await;

                            poke_the_subs::<At>(
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

                            let operations = vec![push_data::Txn {
                                txn_number,
                                operation: data,
                            }];

                            if is_online.read() {
                                Self::prepare_txn_and_send_to_network(
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

    async fn prepare_txn_and_send_to_network(
        sender_to_network: &mut <At::Mpsc as MultiProducerSingleConsumer>::Sender<
            network_actor::MessageToNetwork,
        >,
        operations: Vec<push_data::Txn<push_data::OperationsInput>>,
        state: &cache::State<At::Ch>,
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
            operations1.push(push_data::Txn {
                txn_number: i.txn_number,
                operation: i.operation,
            });
        }

        let t = messages::FromClient {
            jwts,
            nonce: At::Id::generate(),
            operations: operations1,
        };

        let t = At::Ed::encode(&t);

        sender_to_network.send(t).await.unwrap();
    }
}

pub fn collect_subs_to_poke(
    subs_to_poke: &mut HashSet<server_methods::Subscribe>,
    resource: &Vec<ResourceInfo>,
) {
    for resource in resource {
        match resource.resource {
            server_methods::Resource::Jwt(_) => {}
            server_methods::Resource::HashedPassword(_) => todo!(),
            server_methods::Resource::TableUserFieldName(_) => {
                subs_to_poke.insert(server_methods::Subscribe::TableUserFieldName);
            }
            server_methods::Resource::TableUserFieldId(_) => {
                subs_to_poke.insert(server_methods::Subscribe::TableUserFieldId);
            }
            server_methods::Resource::TableCompanyFieldName(_) => {
                subs_to_poke.insert(server_methods::Subscribe::TableCompanyFieldName);
            }
            server_methods::Resource::TableCompanyBranchFieldName(_) => {
                subs_to_poke.insert(server_methods::Subscribe::TableCompanyBranchFieldName);
            }
            server_methods::Resource::TableCompanyBranchFieldCompanyBelong(_) => {
                subs_to_poke
                    .insert(server_methods::Subscribe::TableCompanyBranchFieldCompanyBelong);
            }
            server_methods::Resource::TableCompanyBranchFieldCurrency(_) => {
                subs_to_poke.insert(server_methods::Subscribe::TableCompanyBranchFieldCurrency);
            }
            server_methods::Resource::TableCompanyBranchFieldLocation(_) => {
                subs_to_poke.insert(server_methods::Subscribe::TableCompanyBranchFieldLocation);
            }
            server_methods::Resource::TableCompanyFieldCurrency(_) => {
                subs_to_poke.insert(server_methods::Subscribe::TableCompanyFieldCurrency);
            }
            server_methods::Resource::TableAccessControlForCompanyFieldRole(_) => {
                subs_to_poke
                    .insert(server_methods::Subscribe::TableAccessControlForCompanyFieldRole);
            }
            server_methods::Resource::TableAccessControlForCompanyFieldUser(_) => {
                subs_to_poke
                    .insert(server_methods::Subscribe::TableAccessControlForCompanyFieldUser);
            }
            server_methods::Resource::TableAccessControlForCompanyFieldDataGroup(_) => {
                subs_to_poke
                    .insert(server_methods::Subscribe::TableAccessControlForCompanyFieldDataGroup);
            }
            server_methods::Resource::TableAccessControlForCompanyBranchFieldRole(_) => {
                subs_to_poke
                    .insert(server_methods::Subscribe::TableAccessControlForCompanyBranchFieldRole);
            }
            server_methods::Resource::TableAccessControlForCompanyBranchFieldUser(_) => {
                subs_to_poke
                    .insert(server_methods::Subscribe::TableAccessControlForCompanyBranchFieldUser);
            }
            server_methods::Resource::TableAccessControlForCompanyBranchFieldDataGroup(_) => {
                subs_to_poke.insert(
                    server_methods::Subscribe::TableAccessControlForCompanyBranchFieldDataGroup,
                );
            }
        }
    }
}

async fn poke_the_subs<At: AllClientTypes>(
    pool_of_pokers: &mut HashMap<u16, <At::Mpsc as MultiProducerSingleConsumer>::Sender<()>>,
    pool_of_subscribes: &HashMap<server_methods::Subscribe, HashSet<u16>>,
    subs_to_poke: &HashSet<server_methods::Subscribe>,
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
