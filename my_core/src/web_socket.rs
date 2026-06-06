use crate::prelude::*;

// pub struct Poke;

enum MessageToNetwork {
    Url(String),
    Bytes(Vec<u8>),
}

#[derive(Clone)]
pub struct Response {
    pub is_response_from_server: bool,
    pub data: push_data::OperationsResult,
}

pub struct QueryFromCacheAndServer<Mpsc: MultiProducerSingleConsumer> {
    pub is_submit: bool,
    pub sender: Mpsc::Sender<Option<Response>>,
    pub data: push_data::OperationsInput,
}

pub struct QueryFromCacheOnly<Mpsc: MultiProducerSingleConsumer> {
    pub sender: Mpsc::Sender<cache_query_operations::CacheQueryOutput>,
    pub data: cache_query_operations::CacheQueryInput,
}

enum MessageToCache<Mpsc: MultiProducerSingleConsumer> {
    WeAreBackOnline,
    DataFromServer(Vec<u8>),
    QueryFromCacheAndServer(QueryFromCacheAndServer<Mpsc>),
    QueryFromCacheOnly(QueryFromCacheOnly<Mpsc>),
    // Subscribe {
    //     component_id: u64,
    //     list_of_subscribtion: Vec<server_methods::Subscribe>,
    //     sender_to_component: Mpsc::Sender<Poke>,
    // },
    // UnSubscribe {
    //     component_id: u64,
    // },
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
    pub fn new(sender_to_error: Mpsc::Sender<DynamicError>) -> Self {
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

    pub async fn send_to_cache_actor(
        &self,
        is_submit: bool,
        data: push_data::OperationsInput,
    ) -> Mpsc::Receiver<Option<Response>> {
        let (sender, receiver) = Mpsc::channel();
        self.sender_to_cache
            .lock()
            .unwrap()
            .send(MessageToCache::QueryFromCacheAndServer(
                QueryFromCacheAndServer {
                    is_submit,
                    sender,
                    data,
                },
            ))
            .await
            .unwrap();

        receiver
    }

    pub async fn send_query_to_cache_actor(
        &self,
        msg: cache_query_operations::CacheQueryInput,
    ) -> cache_query_operations::CacheQueryOutput {
        let (sender, mut receiver) = Mpsc::channel();
        self.sender_to_cache
            .lock()
            .unwrap()
            .send(MessageToCache::QueryFromCacheOnly(QueryFromCacheOnly {
                sender,
                data: msg,
            }))
            .await
            .unwrap();

        receiver.recv().await.unwrap()
    }

    pub fn is_online(&self) -> bool {
        get(self.is_online.clone())
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
        mut sender_to_error: Mpsc::Sender<DynamicError>,
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
                        Err(err) => {
                            sender_to_error.send(err).await.unwrap();
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
        mut sender_to_error: Mpsc::Sender<DynamicError>,
        is_online: Arc<RwLock<bool>>,
    ) {
        At::Rt::spawn_local(async move {
            let mut state = cache::State::<At::Ch>::new::<At::Rn>().await;
            let mut pool_of_senders =
                HashMap::<u64, Mpsc::Sender<Option<Response>>>::with_capacity(100);

            loop {
                match receiver_to_cache.recv().await.unwrap() {
                    MessageToCache::WeAreBackOnline => {
                        let operations = state.cache.get_all_txn_input().await;

                        Self::prepare_txn_and_send_to_network(&mut sender_to_network, operations)
                            .await;
                    }
                    MessageToCache::DataFromServer(raw_data) => {
                        let message_type = match At::Ed::decode::<messages::FromServer>(&raw_data) {
                            Ok(message_type) => message_type,
                            Err(err) => {
                                sender_to_error.send(err).await.unwrap();
                                continue;
                            }
                        };

                        match message_type {
                            messages::FromServer::PushData(result) => match result {
                                Ok(results) => {
                                    for txn in results.operations {
                                        state.cache.delete_txn_input(&txn.txn_number).await;
                                        state.cache.write_txn_result(&txn).await;

                                        let sender = pool_of_senders.remove(&txn.txn_number);
                                        if let Some(mut sender) = sender {
                                            let _ = sender
                                                .send(Some(Response {
                                                    is_response_from_server: true,
                                                    data: txn.operation,
                                                }))
                                                .await;
                                            let _ = sender.send(None).await;
                                        }
                                    }

                                    state.state_of_pending_txn = cache::StateOfPendingTxn::new();

                                    let txns = state.cache.get_all_txn_input().await;
                                    for op in txns {
                                        op.operation.run_operation_check_apply(&mut state).await;
                                    }
                                }
                                Err(err) => sender_to_error.send(err.into()).await.unwrap(),
                            },
                            messages::FromServer::Resources(resource) => {
                                state.cache.write_resource(&resource).await;
                            }
                        }
                    }
                    MessageToCache::QueryFromCacheAndServer(QueryFromCacheAndServer {
                        is_submit,
                        mut sender,
                        data,
                    }) => {
                        let txn_number = At::Rn::generate();

                        let result = if is_submit {
                            data.run_operation_check_apply_write(txn_number, &mut state)
                                .await
                        } else {
                            data.run_operation_check(&mut state).await
                        };

                        let _ = sender
                            .send(Some(Response {
                                is_response_from_server: false,
                                data: result,
                            }))
                            .await;

                        let operations = vec![push_data::Txn {
                            txn_number,
                            operation: data,
                        }];

                        if is_submit && get(is_online.clone()) {
                            Self::prepare_txn_and_send_to_network(
                                &mut sender_to_network,
                                operations,
                            )
                            .await;

                            pool_of_senders.insert(txn_number, sender);
                        } else {
                            let _ = sender.send(None).await;
                        };
                    }
                    MessageToCache::QueryFromCacheOnly(op) => {
                        todo!()
                    } // MessageToCache::Subscribe {
                      //     component_id,
                      //     list_of_subscribtion,
                      //     sender_to_component,
                      // } => {
                      // pool_of_senders.insert(component_id, sender_to_component);
                      // for subscribe in list_of_subscribtion {
                      //     pool_of_subscribes
                      //         .entry(subscribe)
                      //         .or_insert(HashSet::with_capacity(10))
                      //         .insert(component_id);
                      // }
                      // }
                      // MessageToCache::UnSubscribe { component_id } => {
                      // pool_of_senders.remove(&component_id);

                      // for (_, component_id_gg) in &mut pool_of_subscribes {
                      //     component_id_gg.remove(&component_id);
                      // }

                      // pool_of_subscribes.retain(|_, component_ids| !component_ids.is_empty());
                      // }
                }
            }
        });
    }

    async fn prepare_txn_and_send_to_network(
        sender_to_network: &mut Mpsc::Sender<MessageToNetwork>,
        operations: Vec<push_data::Txn<push_data::OperationsInput>>,
    ) {
        if operations.is_empty() {
            return;
        }

        let jwts = Vec::new(); // TODO get the jwt that needed for user

        let t = push_data::Input {
            jwts,
            nonce: At::Id::generate().to_row_id(),
            operations,
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
