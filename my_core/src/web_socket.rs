use crate::prelude::*;
use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    time::Duration,
};

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
    fn subscribe(&self, id: u64) -> MyBox<Payload> {
        let box_ = MyBox::new();
        self.0.lock().unwrap().insert(id, box_.clone());
        return box_;
    }

    fn set(&self, id: u64, payload: &Payload) -> Result<(), ()> {
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

    fn unsubscribe(&self, id: u64) {
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
    fn subscribe(&self, path: String) -> MyBox<VecDeque<(u64, Payload)>> {
        let box_ = MyBox::new();
        self.0.lock().unwrap().insert(path, box_.clone());
        return box_;
    }

    fn set(&self, path: String, id: u64, payload: Payload) {
        let box_value = {
            let guard = self.0.lock().unwrap();
            guard.get(&path).cloned()
        };

        if let Some(box_value) = box_value {
            let mut inner_guard = box_value.inner.lock().unwrap();

            let mut queue = match inner_guard.result.clone() {
                Some(queue) => queue,
                None => VecDeque::new(),
            };

            queue.push_back((id, payload));

            inner_guard.result = Some(queue);
            if let Some(waker) = inner_guard.waker.take() {
                waker.wake();
            }
        }
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
    fn subscribe(&self, path: String) -> MyBox<VecDeque<Payload>> {
        let box_ = MyBox::new();
        self.0.lock().unwrap().insert(path, box_.clone());
        return box_;
    }

    fn set(&self, path: String, payload: Payload) {
        let box_value = {
            let guard = self.0.lock().unwrap();
            guard.get(&path).cloned()
        };

        if let Some(box_value) = box_value {
            let mut inner_guard = box_value.inner.lock().unwrap();

            let mut queue = match inner_guard.result.clone() {
                Some(queue) => queue,
                None => VecDeque::new(),
            };

            queue.push_back(payload);

            inner_guard.result = Some(queue);
            if let Some(waker) = inner_guard.waker.take() {
                waker.wake();
            }
        }
    }
}

pub trait WebSocketOp {
    type Error;
    async fn send_bin(&self, data: Vec<u8>) -> Result<(), Self::Error>;
    async fn try_receive_bin(&self) -> Result<Vec<u8>, Self::Error>;
}

pub trait Coding {
    type Error;
    fn encode<T: Serialize>(data: T) -> Vec<u8>;
    fn decode<'de, T: serde::Deserialize<'de>>(data: &'de Vec<u8>) -> Result<T, Self::Error>;
}

pub struct MyClient<WS, DE, RN, RT>
where
    WS: WebSocketOp,
    RT: RuntimeLite,
{
    runtime: PhantomData<RT>,
    random_number: PhantomData<RN>,
    coding: PhantomData<DE>,
    transport: WS,
    send_and_receive_pool: SendAndReceivePool,
    receive_and_send_pool: ReceiveAndSendPool,
    receive_only_pool: ReceiveOnlyPool,
}

impl<WS, DE, E, RN, RT> MyClient<WS, DE, RN, RT>
where
    RN: RandomNumber,
    WS: WebSocketOp<Error = E>,
    DE: Coding<Error = E>,
    RT: RuntimeLite,
{
    pub fn new(transport: WS) -> Self {
        Self {
            runtime: PhantomData::<RT>,
            random_number: PhantomData::<RN>,
            coding: PhantomData::<DE>,
            transport: transport,
            send_and_receive_pool: SendAndReceivePool(Mutex::new(HashMap::new())),
            receive_and_send_pool: ReceiveAndSendPool(Mutex::new(HashMap::new())),
            receive_only_pool: ReceiveOnlyPool(Mutex::new(HashMap::new())),
        }
    }

    pub async fn send_and_receive<SendType: Serialize, ReceiveType: for<'de> Deserialize<'de>>(
        &self,
        path: String,
        payload: SendType,
        timeout_in_secs: u32,
    ) -> Result<ReceiveType, WS::Error> {
        let id = RN::generate();
        let message = self.send_and_receive_pool.subscribe(id);

        let payload = DE::encode(payload);
        let text = MessageType::TwoWay { id, path, payload };
        let text = DE::encode(text);
        self.transport.send_bin(text).await?;

        let r = match RT::timeout_local(Duration::from_secs(timeout_in_secs as u64), message).await
        {
            Ok(result) => DE::decode::<ReceiveType>(&result),
            Err(_) => panic!("noooooooooooooooooooo"),
        };

        self.send_and_receive_pool.unsubscribe(id);
        r
    }

    pub async fn receive_and_send<SendType: Serialize, ReceiveType: for<'de> Deserialize<'de>>(
        &self,
        path: String,
        operation: impl AsyncFn(ReceiveType) -> SendType,
    ) -> Result<(), WS::Error> {
        let message = self.receive_and_send_pool.subscribe(path.clone());

        loop {
            let (id, payload) = message.clone().await;

            let Ok(payload) = DE::decode::<ReceiveType>(&payload) else {
                continue;
            };

            let payload_to_send = operation(payload).await;
            let payload_to_send = DE::encode(payload_to_send);

            let text = MessageType::TwoWay {
                id: id,
                path: path.clone(),
                payload: payload_to_send,
            };
            let text = DE::encode(text);
            self.transport.send_bin(text).await?;
        }
    }

    pub async fn send_only<SendType: Serialize>(
        &self,
        path: String,
        payload: SendType,
    ) -> Result<(), WS::Error> {
        let payload = DE::encode(payload);
        let text = MessageType::OneWay {
            path: path,
            payload: payload,
        };
        let text = DE::encode(text);
        self.transport.send_bin(text).await
    }

    pub async fn receive_only<ReceiveType: for<'de> Deserialize<'de>>(
        &self,
        path: String,
        operation: impl AsyncFn(ReceiveType),
    ) -> Result<(), WS::Error> {
        let message = self.receive_only_pool.subscribe(path.clone());

        loop {
            let payload = message.clone().await;
            let Ok(payload) = DE::decode::<ReceiveType>(&payload) else {
                continue;
            };
            operation(payload).await;
        }
    }

    pub async fn receive_radar(&self) {
        loop {
            let Ok(raw_data) = self.transport.try_receive_bin().await else {
                continue;
            };

            let Ok(decoded_data) = DE::decode::<MessageType>(&raw_data) else {
                continue;
            };

            match decoded_data {
                MessageType::TwoWay { id, path, payload } => {
                    if self.send_and_receive_pool.set(id, &payload).is_err() {
                        self.receive_and_send_pool.set(path, id, payload);
                    };
                }
                MessageType::OneWay { path, payload } => {
                    self.receive_only_pool.set(path, payload);
                }
            }
        }
    }
}
