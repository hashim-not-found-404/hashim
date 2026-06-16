use crate::prelude::*;

enum MessageToNetwork {
    Url(String),
    Bytes(Vec<u8>),
}

#[derive(Clone)]
pub struct Data {
    pub is_response_from_server: bool,
    pub data: operations::Output,
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

pub(crate) enum MessageToCache<Mpsc: MultiProducerSingleConsumer> {
    WeAreBackOnline,
    DataFromServer(Vec<u8>),
    Subscribe {
        component_id: u16,
        list_of_subscribtion: &'static [server_methods::Subscribe],
        sender: Mpsc::Sender<()>,
    },
    UnSubscribe {
        component_id: u16,
    },
    Query {
        strategy: CachingStrategy,
        sender: Mpsc::Sender<Response>,
        data: operations::Input,
    },
}

pub struct MyWAMP<At, Mpsc>
where
    At: AllClientTypes + 'static,
    Mpsc: MultiProducerSingleConsumer + 'static,
{
    _ph: PhantomData<(At, Mpsc)>,
    sender_to_network: Mutex<Mpsc::Sender<MessageToNetwork>>,
    sender_to_cache: Mutex<Mpsc::Sender<MessageToCache<Mpsc>>>,
    is_online: Arc<RwLock<bool>>,
}

impl<At, Mpsc> MyWAMP<At, Mpsc>
where
    At: AllClientTypes + 'static,
    Mpsc: MultiProducerSingleConsumer + 'static,
{
    pub fn new(sender_to_error: Mpsc::Sender<HashimError>) -> Self {
        let (sender_to_network, receiver_to_network) = Mpsc::channel();
        let (sender_to_cache, receiver_to_cache) = Mpsc::channel();

        let is_online = Arc::new(RwLock::new(false));

        Self::network_actor(
            receiver_to_network,
            sender_to_cache.clone(),
            sender_to_error.clone(),
            is_online.clone(),
        );
        Self::cache_actor(
            receiver_to_cache,
            sender_to_network.clone(),
            sender_to_error.clone(),
            is_online.clone(),
        );

        Self {
            _ph: PhantomData,
            sender_to_network: Mutex::new(sender_to_network),
            sender_to_cache: Mutex::new(sender_to_cache),
            is_online,
        }
    }

    pub async fn connect_to_url(&self, url: &String) {
        self.sender_to_network
            .lock()
            .unwrap()
            .send(MessageToNetwork::Url(url.clone()))
            .await
            .unwrap();
    }

    pub(crate) async fn send_to_cache_actor(
        &self,
        strategy: CachingStrategy,
        data: operations::Input,
    ) -> Mpsc::Receiver<Response> {
        let (sender, receiver) = Mpsc::channel();

        self.sender_to_cache
            .lock()
            .unwrap()
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
        &self,
        component_id: u16,
        list_of_subscribtion: &'static [server_methods::Subscribe],
    ) -> Mpsc::Receiver<()> {
        let (sender, receiver) = Mpsc::channel();

        self.sender_to_cache
            .lock()
            .unwrap()
            .send(MessageToCache::Subscribe {
                component_id,
                list_of_subscribtion,
                sender,
            })
            .await
            .unwrap();

        receiver
    }

    pub(crate) async fn send_unsubs_to_cache_actor(&self, component_id: u16) {
        self.sender_to_cache
            .lock()
            .unwrap()
            .send(MessageToCache::UnSubscribe { component_id })
            .await
            .unwrap();
    }

    async fn network_radar(ws: &Option<At::Ws>) -> Result<Vec<u8>, DynamicError> {
        match &ws {
            Some(ws) => ws.receive_bin().await,
            None => Err(HashimError::ConnectionClosed.into()),
        }
    }

    async fn connect(
        is_online: Arc<RwLock<bool>>,
        sender_to_cache: &mut Mpsc::Sender<MessageToCache<Mpsc>>,
        url: &Option<String>,
        ws: &mut Option<At::Ws>,
    ) {
        if let Some(ur) = url {
            set(is_online.clone(), false);

            if let Ok(ok) = At::Ws::connect(ur.as_str()).await {
                *ws = Some(ok);

                sender_to_cache
                    .send(MessageToCache::WeAreBackOnline)
                    .await
                    .unwrap();

                set(is_online.clone(), true);

                return;
            }
        }
        At::Rt::sleep(Duration::from_secs(5)).await;
    }

    fn network_actor(
        mut receiver_to_network: Mpsc::Receiver<MessageToNetwork>,
        mut sender_to_cache: Mpsc::Sender<MessageToCache<Mpsc>>,
        mut sender_to_error: Mpsc::Sender<HashimError>,
        is_online: Arc<RwLock<bool>>,
    ) {
        At::Rt::spawn_local(async move {
            let mut ws: Option<At::Ws> = None;
            let mut url: Option<String> = None;

            loop {
                match At::Rt::select(receiver_to_network.recv(), Self::network_radar(&ws)).await {
                    Either::One(r) => match r.unwrap() {
                        MessageToNetwork::Url(ur) => {
                            url = Some(ur);
                            Self::connect(is_online.clone(), &mut sender_to_cache, &url, &mut ws)
                                .await;
                        }
                        MessageToNetwork::Bytes(data) => match &ws {
                            Some(ws1) => {
                                let result = ws1.send_bin(&data).await;
                                if result.is_err() {
                                    Self::connect(
                                        is_online.clone(),
                                        &mut sender_to_cache,
                                        &url,
                                        &mut ws,
                                    )
                                    .await;
                                }
                            }
                            None => At::Rt::sleep(Duration::from_secs(5)).await,
                        },
                    },
                    Either::Two(from_network) => match from_network {
                        Ok(data) => {
                            sender_to_cache
                                .send(MessageToCache::DataFromServer(data))
                                .await
                                .unwrap();
                        }
                        Err(_) => {
                            sender_to_error
                                .send(HashimError::ConnectionClosed)
                                .await
                                .unwrap();
                            Self::connect(is_online.clone(), &mut sender_to_cache, &url, &mut ws)
                                .await;
                        }
                    },
                }
            }
        });
    }

    fn cache_actor(
        mut receiver_to_cache: Mpsc::Receiver<MessageToCache<Mpsc>>,
        mut sender_to_network: Mpsc::Sender<MessageToNetwork>,
        mut sender_to_error: Mpsc::Sender<HashimError>,
        is_online: Arc<RwLock<bool>>,
    ) {
        At::Rt::spawn_local(async move {
            let mut state = cache::State::<At::Ch>::new::<At::Rn>().await;
            let mut pool_of_senders = HashMap::<u64, Mpsc::Sender<Response>>::with_capacity(100);
            let mut pool_of_pokers = HashMap::<u16, Mpsc::Sender<()>>::with_capacity(10);
            let mut pool_of_subscribes =
                HashMap::<server_methods::Subscribe, HashSet<u16>>::with_capacity(100);

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
                                for txn in results.operations {
                                    let data = txn.operation.map_to_client_output_type();
                                    state.cache.write_resource(&data.extract_resource()).await;

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
                                    op.operation.run_operation_check_apply(&mut state).await;
                                }
                            }
                            messages::FromServer::Resources(resource) => {
                                mbg!(&resource);
                                state.cache.write_resource(&resource).await;

                                let subs_to_poke = what_subs_to_poke(&resource);

                                let mut components_to_poke = HashSet::new();

                                for one_sub in subs_to_poke {
                                    let a = pool_of_subscribes.get(&one_sub);

                                    let a = match a {
                                        Some(a) => a,
                                        None => continue,
                                    };

                                    for a in a {
                                        components_to_poke.insert(a.clone());
                                    }
                                }

                                for i in components_to_poke {
                                    let sender = pool_of_pokers.get_mut(&i).unwrap();
                                    let _ = sender.send(()).await;
                                }
                            }
                        }
                    }
                    MessageToCache::Subscribe {
                        component_id,
                        list_of_subscribtion,
                        sender: sender_to_component,
                    } => {
                        pool_of_pokers.insert(component_id, sender_to_component);
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

                            if get(is_online.clone()) {
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

                            let result = data
                                .run_operation_check_apply_write(txn_number, &mut state)
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

                            if get(is_online.clone()) {
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

    async fn prepare_txn_and_send_to_network<Ch: CacheIO>(
        sender_to_network: &mut Mpsc::Sender<MessageToNetwork>,
        operations: Vec<push_data::Txn<operations::Input>>,
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
            operations1.push(push_data::Txn {
                txn_number: i.txn_number,
                operation: i.operation.map_to_server_input_type(),
            });
        }

        let t = push_data::Input {
            jwts,
            nonce: At::Id::generate().to_uuid(),
            operations: operations1,
        };

        let t = At::Ed::encode(&t);

        sender_to_network
            .send(MessageToNetwork::Bytes(t))
            .await
            .unwrap();
    }
}

fn set<T>(a: Arc<RwLock<T>>, v: T) {
    *a.write().unwrap() = v;
}
fn get<T: Clone>(a: Arc<RwLock<T>>) -> T {
    a.read().unwrap().clone()
}

fn what_subs_to_poke(resource: &Vec<ResourceInfo>) -> HashSet<server_methods::Subscribe> {
    let mut list = HashSet::new();
    for resource in resource {
        match resource.resource {
            server_methods::Resource::Jwt(_) => {}
            server_methods::Resource::UserName(_) => {
                list.insert(server_methods::Subscribe::UserName);
            }
            server_methods::Resource::UserId(_) => {
                list.insert(server_methods::Subscribe::UserId);
            }
            server_methods::Resource::CompanyName(_) => {
                list.insert(server_methods::Subscribe::CompanyName);
            }
            server_methods::Resource::BranchName(_) => {
                list.insert(server_methods::Subscribe::BranchName);
            }
            server_methods::Resource::TableCompanyBranchFieldCompanyBelong(_) => {
                list.insert(server_methods::Subscribe::TableCompanyBranchFieldCompanyBelong);
            }
            server_methods::Resource::CompanyCurrency(_) => {
                list.insert(server_methods::Subscribe::CompanyCurrency);
            }
            server_methods::Resource::RoleAtCompany(_) => {
                list.insert(server_methods::Subscribe::RoleAtCompany);
            }
            server_methods::Resource::UserThatHaveRole(_) => {
                list.insert(server_methods::Subscribe::UserThatHaveRole);
            }
            server_methods::Resource::CompanyThatHaveUserRole(_) => {
                list.insert(server_methods::Subscribe::CompanyThatHaveUserRole);
            }
        }
    }
    list
}
