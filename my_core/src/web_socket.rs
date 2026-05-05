use crate::prelude::*;

pub trait WebSocketOp: Sized {
    async fn connect(url: &str) -> Result<Self, DynamicError>;
    async fn send_bin(&self, data: &Vec<u8>) -> Result<(), DynamicError>;
    async fn receive_bin(&self) -> Result<Vec<u8>, DynamicError>;
}

pub trait Coding {
    fn encode<T: Serialize>(data: &T) -> Vec<u8>;
    fn decode<'de, T: Deserialize<'de>>(data: &'de Vec<u8>) -> Result<T, DynamicError>;
}

pub trait Runtime {
    fn spawn<F: Future + 'static>(fut: F);
    async fn timeout<T, F: Future<Output = T>>(
        duration: Duration,
        fut: F,
    ) -> Result<T, DynamicError>;
}

pub trait WAMP {
    async fn connect(url: &str) -> Result<Arc<Self>, DynamicError>;

    async fn send_and_receive<SendType: Serialize, ReceiveType: for<'de> Deserialize<'de>>(
        self: Arc<Self>,
        path: &String,
        payload: &SendType,
        timeout_in_secs: u32,
    ) -> Result<ReceiveType, DynamicError>;

    async fn receive_and_send<SendType: Serialize, ReceiveType: for<'de> Deserialize<'de>>(
        self: Arc<Self>,
        path: &String,
        operation: impl AsyncFn(ReceiveType) -> SendType,
    ) -> Result<(), DynamicError>;

    async fn send_only<SendType: Serialize>(
        self: Arc<Self>,
        path: &String,
        payload: &SendType,
    ) -> Result<(), DynamicError>;

    async fn receive_only<ReceiveType: for<'de> Deserialize<'de>>(
        self: Arc<Self>,
        path: &String,
        operation: impl AsyncFn(ReceiveType),
    ) -> !;
}

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

#[derive(Clone, Debug)]
struct MyBox<Payload> {
    inner: Arc<Mutex<MyBoxInner<Payload>>>,
}

#[derive(Debug)]
struct MyBoxInner<Payload> {
    result: Option<Payload>,
    waker: Option<Waker>,
}

impl<Payload> MyBox<Payload> {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(MyBoxInner {
                result: None,
                waker: None,
            })),
        }
    }
}

pub struct SendAndReceivePool(Mutex<HashMap<u64, MyBox<Payload>>>);

impl Future for MyBox<Payload> {
    type Output = Payload;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut guard = self.inner.lock().unwrap();

        if let Some(payload) = guard.result.take() {
            return Poll::Ready(payload);
        }

        guard.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

impl SendAndReceivePool {
    fn subscribe(&self, id: &u64) -> MyBox<Payload> {
        let box_ = MyBox::new();
        self.0.lock().unwrap().insert(id.clone(), box_.clone());
        return box_;
    }

    fn set(&self, id: &u64, payload: &Payload) -> Result<(), ()> {
        let guard = self.0.lock().unwrap();
        let option_value = guard.get(&id);

        match option_value {
            Some(box_) => {
                let mut guard = box_.inner.lock().unwrap();
                guard.result = Some(payload.clone());

                if let Some(waker) = guard.waker.take() {
                    waker.wake();
                }
            }
            None => return Err(()),
        }
        Ok(())
    }

    fn unsubscribe(&self, id: &u64) {
        self.0.lock().unwrap().remove(&id);
    }
}

pub struct ReceiveAndSendPool(Mutex<HashMap<String, MyBox<VecDeque<(u64, Payload)>>>>);

impl Future for MyBox<VecDeque<(u64, Payload)>> {
    type Output = (u64, Payload);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut guard = self.inner.lock().unwrap();

        if let Some(payload) = guard.result.as_mut() {
            if let Some(a) = payload.pop_front() {
                guard.waker = Some(cx.waker().clone());
                return Poll::Ready(a);
            }
        }

        guard.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

impl ReceiveAndSendPool {
    fn subscribe(&self, path: &String) -> MyBox<VecDeque<(u64, Payload)>> {
        let box_ = MyBox::new();
        self.0.lock().unwrap().insert(path.clone(), box_.clone());
        return box_;
    }

    fn set(&self, path: &String, id: &u64, payload: &Payload) {
        let box_value = {
            let guard = self.0.lock().unwrap();
            guard.get(path).cloned()
        };

        if let Some(box_value) = box_value {
            let mut inner_guard = box_value.inner.lock().unwrap();

            let mut queue = match inner_guard.result.clone() {
                Some(queue) => queue,
                None => VecDeque::new(),
            };

            queue.push_back((id.clone(), payload.clone()));

            inner_guard.result = Some(queue);
            if let Some(waker) = inner_guard.waker.take() {
                waker.wake();
            }
        }
    }

    fn unsubscribe(&self, path: &String) {
        self.0.lock().unwrap().remove(path);
    }
}

pub struct ReceiveOnlyPool(Mutex<HashMap<String, MyBox<VecDeque<Payload>>>>);

impl Future for MyBox<VecDeque<Payload>> {
    type Output = Payload;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut guard = self.inner.lock().unwrap();

        if let Some(payload) = guard.result.as_mut() {
            if let Some(a) = payload.pop_front() {
                guard.waker = Some(cx.waker().clone());
                return Poll::Ready(a);
            }
        }

        guard.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

impl ReceiveOnlyPool {
    fn subscribe(&self, path: &String) -> MyBox<VecDeque<Payload>> {
        let box_ = MyBox::new();
        self.0.lock().unwrap().insert(path.clone(), box_.clone());
        return box_;
    }

    fn set(&self, path: &String, payload: &Payload) {
        let box_value = {
            let guard = self.0.lock().unwrap();
            guard.get(path).cloned()
        };

        if let Some(box_value) = box_value {
            let mut inner_guard = box_value.inner.lock().unwrap();

            let mut queue = match inner_guard.result.clone() {
                Some(queue) => queue,
                None => VecDeque::new(),
            };

            queue.push_back(payload.clone());

            inner_guard.result = Some(queue);
            if let Some(waker) = inner_guard.waker.take() {
                waker.wake();
            }
        }
    }
}

pub struct MyWAMP<WS, DE, RN, RT>
where
    WS: WebSocketOp,
    RT: Runtime,
{
    runtime: PhantomData<RT>,
    random_number: PhantomData<RN>,
    coding: PhantomData<DE>,
    transport: WS,
    send_and_receive_pool: SendAndReceivePool,
    receive_and_send_pool: ReceiveAndSendPool,
    receive_only_pool: ReceiveOnlyPool,
}

impl<WS, DE, RN, RT> WAMP for MyWAMP<WS, DE, RN, RT>
where
    RN: RandomNumber + 'static,
    WS: WebSocketOp + 'static,
    DE: Coding + 'static,
    RT: Runtime + 'static,
{
    async fn connect(url: &str) -> Result<Arc<Self>, DynamicError> {
        let transport = WS::connect(url).await?;

        let my_client = Arc::new(Self {
            runtime: PhantomData::<RT>,
            random_number: PhantomData::<RN>,
            coding: PhantomData::<DE>,
            transport: transport,
            send_and_receive_pool: SendAndReceivePool(Mutex::new(HashMap::new())),
            receive_and_send_pool: ReceiveAndSendPool(Mutex::new(HashMap::new())),
            receive_only_pool: ReceiveOnlyPool(Mutex::new(HashMap::new())),
        });

        my_client.clone().receive_radar();
        Ok(my_client)
    }

    async fn send_and_receive<SendType: Serialize, ReceiveType: for<'de> Deserialize<'de>>(
        self: Arc<Self>,
        path: &String,
        payload: &SendType,
        timeout_in_secs: u32,
    ) -> Result<ReceiveType, DynamicError> {
        let id = RN::generate();
        let message = self.send_and_receive_pool.subscribe(&id);

        let payload = DE::encode(payload);
        let text = MessageType::TwoWay {
            id,
            path: path.clone(),
            payload,
        };
        let data = DE::encode(&text);

        if let Err(err) = self.transport.send_bin(&data).await {
            self.send_and_receive_pool.unsubscribe(&id);
            return Err(err);
        };

        match RT::timeout(Duration::from_secs(timeout_in_secs as u64), message).await {
            Ok(result) => {
                self.send_and_receive_pool.unsubscribe(&id);
                return DE::decode::<ReceiveType>(&result);
            }
            Err(e) => {
                self.send_and_receive_pool.unsubscribe(&id);
                return Err(e);
            }
        };
    }

    async fn receive_and_send<SendType: Serialize, ReceiveType: for<'de> Deserialize<'de>>(
        self: Arc<Self>,
        path: &String,
        operation: impl AsyncFn(ReceiveType) -> SendType,
    ) -> Result<(), DynamicError> {
        let message = self.receive_and_send_pool.subscribe(path);

        loop {
            let (id, payload) = message.clone().await;

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
            if let Err(err) = self.transport.send_bin(&text).await {
                self.receive_and_send_pool.unsubscribe(path);
                return Err(err);
            };
        }
    }

    async fn send_only<SendType: Serialize>(
        self: Arc<Self>,
        path: &String,
        payload: &SendType,
    ) -> Result<(), DynamicError> {
        let payload = DE::encode(payload);
        let text = MessageType::OneWay {
            path: path.clone(),
            payload: payload,
        };
        let text = DE::encode(&text);
        self.transport.send_bin(&text).await
    }

    async fn receive_only<ReceiveType: for<'de> Deserialize<'de>>(
        self: Arc<Self>,
        path: &String,
        operation: impl AsyncFn(ReceiveType),
    ) -> ! {
        let message = self.receive_only_pool.subscribe(path);

        loop {
            let payload = message.clone().await;
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
    fn receive_radar(self: Arc<Self>) {
        RT::spawn(async move {
            loop {
                let Ok(raw_data) = self.transport.receive_bin().await else {
                    continue;
                };

                let Ok(decoded_data) = DE::decode::<MessageType>(&raw_data) else {
                    continue;
                };

                match decoded_data {
                    MessageType::TwoWay {
                        ref id,
                        ref path,
                        ref payload,
                    } => {
                        if self.send_and_receive_pool.set(id, payload).is_err() {
                            self.receive_and_send_pool.set(path, id, payload);
                        };
                    }
                    MessageType::OneWay {
                        ref path,
                        ref payload,
                    } => {
                        self.receive_only_pool.set(path, payload);
                    }
                }
            }
        });
    }
}
