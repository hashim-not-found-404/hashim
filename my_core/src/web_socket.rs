use crate::prelude::*;
use std::sync::mpsc::{self, Receiver, Sender};

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

enum BrokerMessage {
    FromRadar(Vec<u8>),
    FromSendAndReceive(u64, mpsc::Sender<Payload>),
    FromReceiveAndSend(String, mpsc::Sender<(u64, Payload)>),
    FromReceiveOnly(String, mpsc::Sender<Payload>),
}

pub struct MyWAMP<WS, DE, RN, RT>
where
    WS: WebSocketOp,
    RT: Runtime,
{
    runtime: PhantomData<RT>,
    random_number: PhantomData<RN>,
    coding: PhantomData<DE>,
    transport: PhantomData<WS>,
    broker_mail_box: Sender<BrokerMessage>,
    send_bin_mail_box: Sender<Vec<u8>>,
    error_receiver: Receiver<DynamicError>,
}

impl<WS, DE, RN, RT> MyWAMP<WS, DE, RN, RT>
where
    RN: RandomNumber + 'static,
    WS: WebSocketOp + 'static,
    DE: Coding + 'static,
    RT: Runtime + 'static,
{
}

impl<WS, DE, RN, RT> WAMP for MyWAMP<WS, DE, RN, RT>
where
    RN: RandomNumber + 'static,
    WS: WebSocketOp + 'static,
    DE: Coding + 'static,
    RT: Runtime + 'static,
{
    async fn connect(url: &str) -> Result<Self, DynamicError> {
        let transport = Arc::new(WS::connect(url).await?);

        let (error_mail, error_receiver) = mpsc::channel();

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

    fn get_error(&self) -> DynamicError {
        self.error_receiver.recv().unwrap()
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
        self.send_bin_mail_box.send(data)?;

        let (sender, receiver) = mpsc::channel();
        self.broker_mail_box
            .send(BrokerMessage::FromSendAndReceive(id, sender))?;

        let result = receiver.recv_timeout(Duration::from_secs(timeout_in_secs as u64))?;

        return DE::decode::<ReceiveType>(&result);
    }

    async fn receive_and_send<SendType: Serialize, ReceiveType: for<'de> Deserialize<'de>>(
        &self,
        path: &String,
        operation: impl AsyncFn(ReceiveType) -> SendType,
    ) -> Result<(), DynamicError> {
        let (sender, receiver) = mpsc::channel();
        self.broker_mail_box
            .send(BrokerMessage::FromReceiveAndSend(path.clone(), sender))?;

        loop {
            let (id, payload) = receiver.recv().unwrap();

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
            self.send_bin_mail_box.send(text)?;
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
        self.send_bin_mail_box.send(text)?;
        Ok(())
    }

    async fn receive_only<ReceiveType: for<'de> Deserialize<'de>>(
        &self,
        path: &String,
        operation: impl AsyncFn(ReceiveType),
    ) -> ! {
        let (sender, receiver) = mpsc::channel();
        self.broker_mail_box
            .send(BrokerMessage::FromReceiveOnly(path.clone(), sender))
            .unwrap();

        loop {
            let payload = receiver.recv().unwrap();
            let Ok(payload) = DE::decode::<ReceiveType>(&payload) else {
                continue;
            };
            operation(payload).await;
        }
    }
}

impl<WS, DE, RN, RT> MyWAMP<WS, DE, RN, RT>
where
    RN: RandomNumber + 'static,
    WS: WebSocketOp + 'static,
    DE: Coding + 'static,
    RT: Runtime + 'static,
{
    fn broker_actor(error_mail_box: Sender<DynamicError>) -> Sender<BrokerMessage> {
        let (sender, receiver) = mpsc::channel();

        RT::spawn(async move {
            let mut send_and_receive_pool = HashMap::<u64, mpsc::Sender<Payload>>::new();
            let mut receive_and_send_pool = HashMap::<String, mpsc::Sender<(u64, Payload)>>::new();
            let mut receive_only_pool = HashMap::<String, mpsc::Sender<Payload>>::new();

            loop {
                match receiver.recv().unwrap() {
                    BrokerMessage::FromRadar(raw_data) => {
                        let message_type = match DE::decode::<MessageType>(&raw_data) {
                            Ok(message_type) => message_type,
                            Err(err) => {
                                error_mail_box.send(err).unwrap();
                                continue;
                            }
                        };

                        match message_type {
                            MessageType::TwoWay { id, path, payload } => {
                                if let Some(sender) = send_and_receive_pool.remove(&id) {
                                    sender.send(payload).unwrap();
                                    continue;
                                }

                                if let Some(sender) = receive_and_send_pool.get(&path) {
                                    sender.send((id, payload)).unwrap();
                                    continue;
                                }

                                error_mail_box
                                .send(
                                    "there is data received in wrong path or maybe timeout there was".into(),
                                )
                                .unwrap();
                            }
                            MessageType::OneWay { path, payload } => {
                                if let Some(sender) = receive_only_pool.get(&path) {
                                    sender.send(payload).unwrap();
                                    continue;
                                }

                                error_mail_box
                                    .send("there is data received in wrong path".into())
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
        broker_mail_box: Sender<BrokerMessage>,
        error_mail_box: Sender<DynamicError>,
    ) {
        RT::spawn(async move {
            loop {
                match transport.receive_bin().await {
                    Ok(data) => {
                        broker_mail_box
                            .send(BrokerMessage::FromRadar(data))
                            .unwrap();
                    }

                    Err(err) => {
                        error_mail_box.send(err).unwrap();
                    }
                }
            }
        })
    }

    fn send_bin_actor(transport: Arc<WS>, error_mail_box: Sender<DynamicError>) -> Sender<Vec<u8>> {
        let (sender, receiver) = mpsc::channel();

        let sender1 = sender.clone();
        RT::spawn(async move {
            loop {
                let bin = receiver.recv().unwrap();
                if let Err(err) = transport.send_bin(&bin).await {
                    sender1.send(bin).unwrap();
                    error_mail_box.send(err).unwrap();
                    RT::sleep(Duration::from_secs(1)).await;
                };
            }
        });

        sender
    }
}
