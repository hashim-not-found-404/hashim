use crate::prelude::*;

type Payload = Vec<u8>;

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    TwoWay {
        id: u64,
        path: String,
        payload: Payload,
    },
    OneWay {
        path: String,
        payload: Payload,
    },
}

enum MessageToConnector {
    ShutDown,
    Url(String),
    Reconnect,
}

enum MessageToBroker<MPSC: MultiProducerSingleConsumer> {
    ShutDown,
    FromRadar(Vec<u8>),
    FromSendAndReceive(u64, MPSC::Sender<Payload>),
    FromReceiveAndSend(String, MPSC::Sender<(u64, Payload)>),
    FromReceiveOnly(String, MPSC::Sender<Payload>),
}

enum MessageToReceiveBin {
    ShutDown,
}

enum MessageToSendBin {
    ShutDown,
    Bin(Vec<u8>),
}

pub struct MyWAMP<WS, DE, RN, RT, MPSC>
where
    RN: RandomNumber + 'static,
    WS: WebSocketOp + 'static,
    DE: Coding + 'static,
    RT: Runtime + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
{
    runtime: PhantomData<RT>,
    random_number: PhantomData<RN>,
    coding: PhantomData<DE>,
    transport: PhantomData<WS>,

    sender_to_connector: MPSC::Sender<MessageToConnector>,
    sender_to_broker: MPSC::Sender<MessageToBroker<MPSC>>,
    sender_to_send_bin: MPSC::Sender<MessageToSendBin>,
    sender_to_receive_bin: MPSC::Sender<MessageToReceiveBin>,
}

impl<WS, DE, RN, RT, MPSC> WAMP for MyWAMP<WS, DE, RN, RT, MPSC>
where
    RN: RandomNumber + 'static,
    WS: WebSocketOp + 'static,
    DE: Coding + 'static,
    RT: Runtime + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
{
    type Sender<T> = MPSC::Sender<T>;

    fn new(sender_to_error: Self::Sender<DynamicError>) -> Self {
        let (sender_to_connector, receiver_to_connector) = MPSC::channel();
        let (sender_to_broker, receiver_to_broker) = MPSC::channel();
        let (sender_to_send_bin, receiver_to_send_bin) = MPSC::channel();
        let (sender_to_receive_bin, receiver_to_receive_bin) = MPSC::channel();

        let ws = Arc::new(RwLock::new(None));

        Self::connector_actor(ws.clone(), receiver_to_connector, sender_to_error.clone());
        Self::broker_actor(receiver_to_broker, sender_to_error.clone());
        Self::receive_bin_actor(
            ws.clone(),
            receiver_to_receive_bin,
            sender_to_connector.clone(),
            sender_to_broker.clone(),
            sender_to_error.clone(),
        );
        Self::send_bin_actor(
            ws.clone(),
            receiver_to_send_bin,
            sender_to_send_bin.clone(),
            sender_to_connector.clone(),
            sender_to_error.clone(),
        );

        Self {
            runtime: PhantomData::<RT>,
            random_number: PhantomData::<RN>,
            coding: PhantomData::<DE>,
            transport: PhantomData::<WS>,
            sender_to_connector,
            sender_to_broker,
            sender_to_send_bin,
            sender_to_receive_bin,
        }
    }

    async fn connect_to_url(&self, url: &String) {
        self.sender_to_connector
            .send(MessageToConnector::Url(url.clone()))
            .await
            .unwrap();
    }

    async fn close(self) {
        self.sender_to_connector
            .send(MessageToConnector::ShutDown)
            .await
            .unwrap();
        self.sender_to_broker
            .send(MessageToBroker::ShutDown)
            .await
            .unwrap();
        self.sender_to_receive_bin
            .send(MessageToReceiveBin::ShutDown)
            .await
            .unwrap();
        self.sender_to_send_bin
            .send(MessageToSendBin::ShutDown)
            .await
            .unwrap();
    }

    async fn send_and_receive<SendType: Serialize, ReceiveType: for<'de> Deserialize<'de>>(
        &self,
        path: &String,
        payload: &SendType,
        timeout_in_secs: u32,
    ) -> Result<ReceiveType, DynamicError> {
        let id = RN::generate();

        let payload = DE::encode(payload);
        let text = MessageType::TwoWay {
            id,
            path: path.clone(),
            payload,
        };
        let data = DE::encode(&text);
        self.sender_to_send_bin
            .send(MessageToSendBin::Bin(data))
            .await?;

        let (sender, receiver) = MPSC::channel();
        self.sender_to_broker
            .send(MessageToBroker::<MPSC>::FromSendAndReceive(id, sender))
            .await?;

        let result =
            RT::timeout(Duration::from_secs(timeout_in_secs as u64), receiver.recv()).await??;

        return DE::decode::<ReceiveType>(&result);
    }

    async fn receive_and_send<SendType: Serialize, ReceiveType: for<'de> Deserialize<'de>>(
        &self,
        path: &String,
        operation: impl AsyncFn(ReceiveType) -> SendType,
    ) -> Result<(), DynamicError> {
        let (sender, receiver) = MPSC::channel();
        self.sender_to_broker
            .send(MessageToBroker::<MPSC>::FromReceiveAndSend(
                path.clone(),
                sender,
            ))
            .await?;

        loop {
            let (id, payload) = receiver.recv().await.unwrap();

            let Ok(payload) = DE::decode::<ReceiveType>(&payload) else {
                continue;
            };

            let payload_to_send = operation(payload).await;
            let payload_to_send = DE::encode(&payload_to_send);

            let text = MessageType::TwoWay {
                id: id,
                path: path.clone(),
                payload: payload_to_send,
            };
            let text = DE::encode(&text);
            self.sender_to_send_bin
                .send(MessageToSendBin::Bin(text))
                .await?;
        }
    }

    async fn send_only<SendType: Serialize>(
        &self,
        path: &String,
        payload: &SendType,
    ) -> Result<(), DynamicError> {
        let payload = DE::encode(payload);
        let text = MessageType::OneWay {
            path: path.clone(),
            payload: payload,
        };
        let text = DE::encode(&text);
        self.sender_to_send_bin
            .send(MessageToSendBin::Bin(text))
            .await?;
        Ok(())
    }

    async fn receive_only<ReceiveType: for<'de> Deserialize<'de>>(
        &self,
        path: &String,
        operation: impl AsyncFn(ReceiveType) + 'static,
    ) {
        let (sender, receiver) = MPSC::channel();
        self.sender_to_broker
            .send(MessageToBroker::<MPSC>::FromReceiveOnly(
                path.clone(),
                sender,
            ))
            .await
            .unwrap();

        RT::spawn(async move {
            loop {
                let payload = receiver.recv().await.unwrap();
                let Ok(payload) = DE::decode::<ReceiveType>(&payload) else {
                    continue;
                };
                operation(payload).await;
            }
        })
    }
}

impl<WS, DE, RN, RT, MPSC> MyWAMP<WS, DE, RN, RT, MPSC>
where
    RN: RandomNumber + 'static,
    WS: WebSocketOp + 'static,
    DE: Coding + 'static,
    RT: Runtime + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
{
    fn connector_actor(
        ws: Arc<RwLock<Option<WS>>>,
        receiver_to_connector: MPSC::Receiver<MessageToConnector>,
        sender_to_error: MPSC::Sender<DynamicError>,
    ) {
        RT::spawn(async move {
            let mut url = None;
            loop {
                match receiver_to_connector.recv().await.unwrap() {
                    MessageToConnector::ShutDown => return,
                    MessageToConnector::Url(ur) => url = Some(ur),
                    MessageToConnector::Reconnect => {}
                };

                if let Some(ref ur) = url {
                    let result = WS::connect(ur.as_str()).await;
                    match result {
                        Ok(o) => {
                            let mut guard = ws.write().unwrap();
                            *guard = Some(o);
                        }
                        Err(e) => sender_to_error.send(e).await.unwrap(),
                    }
                }
            }
        })
    }

    fn broker_actor(
        receiver_to_broker: MPSC::Receiver<MessageToBroker<MPSC>>,
        sender_to_error: MPSC::Sender<DynamicError>,
    ) {
        RT::spawn(async move {
            let mut send_and_receive_pool = HashMap::<u64, MPSC::Sender<Payload>>::new();
            let mut receive_and_send_pool = HashMap::<String, MPSC::Sender<(u64, Payload)>>::new();
            let mut receive_only_pool = HashMap::<String, MPSC::Sender<Payload>>::new();

            loop {
                match receiver_to_broker.recv().await.unwrap() {
                    MessageToBroker::FromRadar(raw_data) => {
                        let message_type = match DE::decode::<MessageType>(&raw_data) {
                            Ok(message_type) => message_type,
                            Err(err) => {
                                sender_to_error.send(err).await.unwrap();
                                continue;
                            }
                        };

                        match message_type {
                            MessageType::TwoWay { id, path, payload } => {
                                if let Some(sender) = send_and_receive_pool.remove(&id) {
                                    if let Ok(_) = sender.send(payload.clone()).await {
                                        continue;
                                    }

                                    if let Some(sender) = receive_only_pool.get_mut(&path) {
                                        sender.send(payload).await.unwrap();
                                        continue;
                                    }

                                    continue;
                                }

                                if let Some(sender) = receive_and_send_pool.get_mut(&path) {
                                    sender.send((id, payload)).await.unwrap();
                                    continue;
                                }

                                sender_to_error
                                .send(
                                    "there is data received in wrong path or maybe timeout there was".into(),
                                )
                                .await
                                .unwrap();
                            }
                            MessageType::OneWay { path, payload } => {
                                if let Some(sender) = receive_only_pool.get_mut(&path) {
                                    sender.send(payload).await.unwrap();
                                    continue;
                                }

                                sender_to_error
                                    .send("there is data received in wrong path".into())
                                    .await
                                    .unwrap();
                            }
                        }
                    }
                    MessageToBroker::FromSendAndReceive(id, sender) => {
                        send_and_receive_pool.insert(id, sender);
                    }
                    MessageToBroker::FromReceiveAndSend(path, sender) => {
                        receive_and_send_pool.insert(path, sender);
                    }
                    MessageToBroker::FromReceiveOnly(path, sender) => {
                        receive_only_pool.insert(path, sender);
                    }
                    MessageToBroker::ShutDown => return,
                }
            }
        })
    }

    fn receive_bin_actor(
        ws: Arc<RwLock<Option<WS>>>,
        receiver_to_receive_bin: MPSC::Receiver<MessageToReceiveBin>,
        sender_to_connector: MPSC::Sender<MessageToConnector>, // TODO : i need to send reconnect
        sender_to_broker: MPSC::Sender<MessageToBroker<MPSC>>,
        sender_to_error: MPSC::Sender<DynamicError>,
    ) {
        RT::spawn(async move {
            loop {
                let fut1 = async {
                    let guard = ws.read().unwrap();
                    let guard = guard.as_ref();

                    match guard {
                        Some(ws) => ws.receive_bin().await,
                        None => Err("error he is at init".into()),
                    }
                };

                match RT::select(fut1, receiver_to_receive_bin.recv()).await {
                    Either::Left(l) => match l {
                        Ok(data) => {
                            sender_to_broker
                                .send(MessageToBroker::FromRadar(data))
                                .await
                                .unwrap();
                        }

                        Err(err) => {
                            sender_to_error.send(err).await.unwrap();
                        }
                    },
                    Either::Right(r) => {
                        r.unwrap();
                        return;
                    }
                }
            }
        })
    }

    fn send_bin_actor(
        ws: Arc<RwLock<Option<WS>>>,
        receiver_to_send_bin: MPSC::Receiver<MessageToSendBin>,
        sender_to_send_bin: MPSC::Sender<MessageToSendBin>,
        sender_to_connector: MPSC::Sender<MessageToConnector>,
        sender_to_error: MPSC::Sender<DynamicError>,
    ) {
        RT::spawn(async move {
            loop {
                match receiver_to_send_bin.recv().await.unwrap() {
                    MessageToSendBin::ShutDown => return,
                    MessageToSendBin::Bin(bin) => {
                        let guard = ws.read().unwrap();
                        let guard = guard.as_ref();

                        match guard {
                            Some(ws) => {
                                if let Err(err) = ws.send_bin(&bin).await {
                                    sender_to_send_bin
                                        .send(MessageToSendBin::Bin(bin))
                                        .await
                                        .unwrap();
                                    sender_to_error.send(err).await.unwrap();
                                    RT::sleep(Duration::from_secs(1)).await;
                                }
                            }

                            None => {
                                sender_to_send_bin
                                    .send(MessageToSendBin::Bin(bin))
                                    .await
                                    .unwrap();

                                RT::sleep(Duration::from_secs(1)).await;
                            }
                        };
                    }
                }
            }
        })
    }
}
