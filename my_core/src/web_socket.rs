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

enum BrokerMessage<MPSC: MultiProducerSingleConsumer> {
    FromRadar(Vec<u8>),
    FromSendAndReceive(u64, MPSC::Sender<Payload>),
    FromReceiveAndSend(String, MPSC::Sender<(u64, Payload)>),
    FromReceiveOnly(String, MPSC::Sender<Payload>),
}

pub struct MyWAMP<WS, DE, RN, RT, MPSC>
where
    MPSC: MultiProducerSingleConsumer,
{
    runtime: PhantomData<RT>,
    random_number: PhantomData<RN>,
    coding: PhantomData<DE>,
    transport: PhantomData<WS>,
    broker_mail_box: MPSC::Sender<BrokerMessage<MPSC>>,
    send_bin_mail_box: MPSC::Sender<Vec<u8>>,
    error_receiver: MPSC::Receiver<DynamicError>,
}

impl<WS, DE, RN, RT, MPSC> WAMP for MyWAMP<WS, DE, RN, RT, MPSC>
where
    RN: RandomNumber + 'static,
    WS: WebSocketOp + 'static,
    DE: Coding + 'static,
    RT: Runtime + 'static,
    MPSC: MultiProducerSingleConsumer + 'static,
{
    // TODO : i need to send the error to the error actor and implement reconnect or (connecting actor)
    async fn connect(url: &str) -> Result<Self, DynamicError> {
        let transport = Arc::new(WS::connect(url).await?);

        let (error_mail, error_receiver) = MPSC::channel();

        let broker_mail = Self::broker_actor(error_mail.clone());
        Self::receive_bin_actor(transport.clone(), broker_mail.clone(), error_mail.clone());
        let send_bin_mail = Self::send_bin_actor(transport.clone(), error_mail.clone());

        let my_client = Self {
            runtime: PhantomData::<RT>,
            random_number: PhantomData::<RN>,
            coding: PhantomData::<DE>,
            transport: PhantomData::<WS>,
            broker_mail_box: broker_mail,
            send_bin_mail_box: send_bin_mail,
            error_receiver: error_receiver,
        };

        Ok(my_client)
    }

    async fn get_error(&self) -> DynamicError {
        self.error_receiver.recv().await.unwrap()
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
        self.send_bin_mail_box.send(data).await?;

        let (sender, receiver) = MPSC::channel();
        self.broker_mail_box
            .send(BrokerMessage::<MPSC>::FromSendAndReceive(id, sender))
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
        self.broker_mail_box
            .send(BrokerMessage::<MPSC>::FromReceiveAndSend(
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
            self.send_bin_mail_box.send(text).await?;
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
        self.send_bin_mail_box.send(text).await?;
        Ok(())
    }

    async fn receive_only<ReceiveType: for<'de> Deserialize<'de>>(
        &self,
        path: &String,
        operation: impl AsyncFn(ReceiveType),
    ) -> ! {
        let (sender, receiver) = MPSC::channel();
        self.broker_mail_box
            .send(BrokerMessage::<MPSC>::FromReceiveOnly(path.clone(), sender))
            .await
            .unwrap();

        loop {
            let payload = receiver.recv().await.unwrap();
            let Ok(payload) = DE::decode::<ReceiveType>(&payload) else {
                continue;
            };
            operation(payload).await;
        }
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
    fn broker_actor(
        error_mail_box: MPSC::Sender<DynamicError>,
    ) -> MPSC::Sender<BrokerMessage<MPSC>> {
        let (sender, receiver) = MPSC::channel();

        RT::spawn(async move {
            let mut send_and_receive_pool = HashMap::<u64, MPSC::Sender<Payload>>::new();
            let mut receive_and_send_pool = HashMap::<String, MPSC::Sender<(u64, Payload)>>::new();
            let mut receive_only_pool = HashMap::<String, MPSC::Sender<Payload>>::new();

            loop {
                match receiver.recv().await.unwrap() {
                    BrokerMessage::FromRadar(raw_data) => {
                        let message_type = match DE::decode::<MessageType>(&raw_data) {
                            Ok(message_type) => message_type,
                            Err(err) => {
                                error_mail_box.send(err).await.unwrap();
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

                                error_mail_box
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

                                error_mail_box
                                    .send("there is data received in wrong path".into())
                                    .await
                                    .unwrap();
                            }
                        }
                    }
                    BrokerMessage::FromSendAndReceive(id, sender) => {
                        send_and_receive_pool.insert(id, sender);
                    }
                    BrokerMessage::FromReceiveAndSend(path, sender) => {
                        receive_and_send_pool.insert(path, sender);
                    }
                    BrokerMessage::FromReceiveOnly(path, sender) => {
                        receive_only_pool.insert(path, sender);
                    }
                }
            }
        });

        sender
    }

    fn receive_bin_actor(
        transport: Arc<WS>,
        broker_mail_box: MPSC::Sender<BrokerMessage<MPSC>>,
        error_mail_box: MPSC::Sender<DynamicError>,
    ) {
        RT::spawn(async move {
            loop {
                match transport.receive_bin().await {
                    Ok(data) => {
                        broker_mail_box
                            .send(BrokerMessage::FromRadar(data))
                            .await
                            .unwrap();
                    }

                    Err(err) => {
                        error_mail_box.send(err).await.unwrap();
                    }
                }
            }
        })
    }

    fn send_bin_actor(
        transport: Arc<WS>,
        error_mail_box: MPSC::Sender<DynamicError>,
    ) -> MPSC::Sender<Vec<u8>> {
        let (sender, receiver) = MPSC::channel();

        let sender1 = sender.clone();
        RT::spawn(async move {
            loop {
                let bin = receiver.recv().await.unwrap();
                if let Err(err) = transport.send_bin(&bin).await {
                    sender1.send(bin).await.unwrap();
                    error_mail_box.send(err).await.unwrap();
                    RT::sleep(Duration::from_secs(1)).await;
                };
            }
        });

        sender
    }
}
