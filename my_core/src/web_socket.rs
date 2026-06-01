use crate::prelude::*;

pub struct Poke;

enum MessageToNetwork {
    ShutDown,
    Url(String),
    Bytes(Vec<u8>),
}

pub struct Query<MPSC: MultiProducerSingleConsumer> {
    pub sender: MPSC::Sender<push_data::OperationsResult>,
    pub data: push_data::OperationsInput,
}

enum MessageToCache<MPSC: MultiProducerSingleConsumer> {
    ShutDown,
    WeAreOnline,
    WeAreOffline,
    DataFromServer(Vec<u8>),
    Query(Query<MPSC>),
    Subscribe {
        component_id: u64,
        list_of_subscribtion: Vec<server_methods::Subscribe>,
        sender_to_component: MPSC::Sender<Poke>,
    },
    UnSubscribe {
        component_id: u64,
    },
}

pub struct MyWAMP<WS, DE, RN, RT, CH, Id, MPSC>
where
    RN: RandomNumber + 'static,
    WS: WebSocketOp + 'static,
    DE: Coding + 'static,
    RT: Runtime + 'static,
    CH: CacheIO + 'static,
    Id: RowId + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
{
    _ph: PhantomData<(WS, DE, RN, RT, CH, Id, MPSC)>,
    sender_to_network: MPSC::Sender<MessageToNetwork>,
    sender_to_cache: MPSC::Sender<MessageToCache<MPSC>>,
}

impl<WS, DE, RN, RT, CH, Id, MPSC> MyWAMP<WS, DE, RN, RT, CH, Id, MPSC>
where
    RN: RandomNumber + 'static,
    WS: WebSocketOp + 'static,
    DE: Coding + 'static,
    RT: Runtime + 'static,
    CH: CacheIO + 'static,
    Id: RowId + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
{
    pub fn new(sender_to_error: MPSC::Sender<DynamicError>) -> Self {
        let (sender_to_network, receiver_to_network) = MPSC::channel();
        let (sender_to_cache, receiver_to_cache) = MPSC::channel();

        Self::network_actor(
            receiver_to_network,
            sender_to_cache.clone(),
            sender_to_error.clone(),
        );
        Self::cache_actor(
            receiver_to_cache,
            sender_to_network.clone(),
            sender_to_error.clone(),
        );

        Self {
            _ph: PhantomData,
            sender_to_network,
            sender_to_cache,
        }
    }

    pub async fn connect_to_url(&self, url: &String) {
        self.sender_to_network
            .send(MessageToNetwork::Url(url.clone()))
            .await
            .unwrap();
    }

    pub async fn close(self) {
        self.sender_to_network
            .send(MessageToNetwork::ShutDown)
            .await
            .unwrap();
        self.sender_to_cache
            .send(MessageToCache::ShutDown)
            .await
            .unwrap();
    }

    pub async fn send_to_cache_actor(&self, msg: Query<MPSC>) {
        self.sender_to_cache
            .send(MessageToCache::Query(msg))
            .await
            .unwrap();
    }

    async fn network_radar(ws: &Option<WS>) -> Result<Vec<u8>, DynamicError> {
        match &ws {
            Some(ws) => ws.receive_bin().await,
            None => Err(HashimError::ConnectionClosed.into()),
        }
    }

    async fn connect(
        sender_to_cache: &MPSC::Sender<MessageToCache<MPSC>>,
        url: &Option<String>,
        ws: &mut Option<WS>,
    ) {
        if let Some(ur) = url {
            sender_to_cache
                .send(MessageToCache::WeAreOffline)
                .await
                .unwrap();

            if let Ok(ok) = WS::connect(ur.as_str()).await {
                *ws = Some(ok);
                sender_to_cache
                    .send(MessageToCache::WeAreOnline)
                    .await
                    .unwrap();

                return;
            }
        }
        RT::sleep(Duration::from_secs(5)).await;
    }

    fn network_actor(
        receiver_to_network: MPSC::Receiver<MessageToNetwork>,
        sender_to_cache: MPSC::Sender<MessageToCache<MPSC>>,
        sender_to_error: MPSC::Sender<DynamicError>,
    ) {
        RT::spawn_local(async move {
            let mut ws: Option<WS> = None;
            let mut url: Option<String> = None;

            loop {
                match RT::select(receiver_to_network.recv(), Self::network_radar(&ws)).await {
                    Either::One(r) => match r.unwrap() {
                        MessageToNetwork::ShutDown => return,
                        MessageToNetwork::Url(ur) => {
                            url = Some(ur);
                            Self::connect(&sender_to_cache, &url, &mut ws).await;
                        }
                        MessageToNetwork::Bytes(data) => match &ws {
                            Some(ws1) => {
                                let result = ws1.send_bin(&data).await;
                                if result.is_err() {
                                    Self::connect(&sender_to_cache, &url, &mut ws).await;
                                }
                            }
                            None => RT::sleep(Duration::from_secs(5)).await,
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
                            Self::connect(&sender_to_cache, &url, &mut ws).await;
                        }
                    },
                }
            }
        })
    }

    fn cache_actor(
        receiver_to_cache: MPSC::Receiver<MessageToCache<MPSC>>,
        sender_to_network: MPSC::Sender<MessageToNetwork>,
        sender_to_error: MPSC::Sender<DynamicError>,
    ) {
        RT::spawn_local(async move {
            let mut state = cache::State::<CH>::new::<RN>().await;

            let mut is_online = false;
            let mut pool_of_subscribes =
                HashMap::<server_methods::Subscribe, HashSet<u64>>::with_capacity(1000);
            let mut pool_of_senders = HashMap::<u64, MPSC::Sender<Poke>>::with_capacity(1000);

            loop {
                match receiver_to_cache.recv().await.unwrap() {
                    MessageToCache::ShutDown => return,
                    MessageToCache::WeAreOnline => {
                        is_online = true;

                        let operations = state.cache.get_all_txn_input().await;

                        if operations.is_empty() {
                            continue;
                        }

                        let data = push_data::Input {
                            jwts: Vec::new(), // TODO
                            nonce: Id::generate().to_string(),
                            operations,
                        };

                        let data = DE::encode(&data);

                        sender_to_network
                            .send(MessageToNetwork::Bytes(data))
                            .await
                            .unwrap();
                    }
                    MessageToCache::WeAreOffline => is_online = false,
                    MessageToCache::DataFromServer(raw_data) => {
                        let message_type = match DE::decode::<messages::FromServer>(&raw_data) {
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
                                    }

                                    let txns = state.cache.get_all_txn_input().await;

                                    state.state_of_pending_txn = cache::StateOfPendingTxn::new();

                                    for op in txns {
                                        op.operation
                                            .run_operation::<_, RN>(&mut state, false)
                                            .await;
                                    }
                                }
                                Err(err) => sender_to_error.send(err.into()).await.unwrap(),
                            },
                            messages::FromServer::Resources(resource_infos) => {
                                todo!("TODO update the pub/sub")
                            }
                        }
                    }
                    MessageToCache::Query(Query { sender, data }) => {
                        let result = data.run_operation::<_, RN>(&mut state, true).await;

                        let _ = sender.send(result).await;

                        let txn = push_data::Txn {
                            txn_number: RN::generate(),
                            operation: data,
                        };

                        let operations = vec![txn];

                        if is_online {
                            Self::prepare_txn_and_send_to_network(
                                sender_to_network.clone(),
                                Vec::new(),
                                operations,
                            )
                            .await;
                        };
                    }
                    MessageToCache::Subscribe {
                        component_id,
                        list_of_subscribtion,
                        sender_to_component,
                    } => {
                        pool_of_senders.insert(component_id, sender_to_component);
                        for subscribe in list_of_subscribtion {
                            pool_of_subscribes
                                .entry(subscribe)
                                .or_insert(HashSet::with_capacity(10))
                                .insert(component_id);
                        }
                    }
                    MessageToCache::UnSubscribe { component_id } => {
                        pool_of_senders.remove(&component_id);

                        for (_, component_id_gg) in &mut pool_of_subscribes {
                            component_id_gg.remove(&component_id);
                        }

                        pool_of_subscribes.retain(|_, component_ids| !component_ids.is_empty());
                    }
                }
            }
        })
    }

    async fn prepare_txn_and_send_to_network(
        sender_to_network: MPSC::Sender<MessageToNetwork>,
        jwts: Vec<String>,
        operations: Vec<push_data::Txn<push_data::OperationsInput>>,
    ) {
        let t = push_data::Input {
            jwts,
            nonce: Id::generate().to_string(),
            operations,
        };

        let t = DE::encode(&t);

        sender_to_network
            .send(MessageToNetwork::Bytes(t))
            .await
            .unwrap();
    }
}
