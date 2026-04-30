#[cfg(not(target_arch = "wasm32"))]
use agnostic_lite::tokio::{TokioRuntime as NativeRuntime, TokioSpawner as NativeSpawner};

#[cfg(target_arch = "wasm32")]
use agnostic_lite::wasm::{WasmRuntime as NativeRuntime, WasmSpawner as NativeSpawner};

use agnostic_lite::AsyncLocalSpawner;
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use impls_for_wasm::a1::RandomNumberS;
use my_core::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio_tungstenite_wasm::{Message, WebSocketStream, connect};

#[derive(Debug, Clone)]
pub enum MyError {
    DecodingError,
    Closed,
    ServerError,
    Timeout,
    //
    OtherUnexpectedStatusCode(String),
    SomeInternalErrorOfTheServer,
    Decoding,
    CheckYourWifi,
    ErrorAtSendingRequest(String),
}

impl ToString for MyError {
    fn to_string(&self) -> String {
        match self {
            Self::OtherUnexpectedStatusCode(_) => {
                return String::from("other_unexpected_status_code");
            }
            Self::SomeInternalErrorOfTheServer => {
                return String::from("some_internal_error_of_the_server");
            }
            Self::Decoding => {
                return String::from("decoding");
            }
            Self::CheckYourWifi => {
                return String::from("check_your_wifi");
            }
            Self::ErrorAtSendingRequest(s) => {
                return String::from("error_at_sending_request");
            }
            MyError::DecodingError => todo!(),
            MyError::Closed => todo!(),
            MyError::ServerError => todo!(),
            MyError::Timeout => String::from("Timeout"),
        }
    }
}

struct Atooooooooooo;
impl my_core::web_socket::Coding for Atooooooooooo {
    type Error = MyError;

    fn encode<T: Serialize>(data: T) -> Vec<u8> {
        use postcard::to_allocvec;
        to_allocvec(&data).unwrap().to_vec()
    }

    fn decode<'de, T: Deserialize<'de>>(data: &'de Vec<u8>) -> Result<T, Self::Error> {
        use postcard::from_bytes;
        match from_bytes::<T>(data) {
            Ok(text) => return Ok(text),
            Err(_) => return Err(MyError::DecodingError),
        }
    }
}

pub struct MyClient {
    write: Mutex<SplitSink<WebSocketStream, Message>>,
    read: Mutex<SplitStream<WebSocketStream>>,
}

impl MyClient {
    pub async fn new() -> Self {
        let url = format!("ws://{}/ws", ADDRESS);

        let ws_stream = connect(url).await;

        let ws_stream = match ws_stream {
            Ok(o) => o,
            Err(e) => {
                panic!("{:?}", e)
            }
        };

        let (write, read) = ws_stream.split();

        Self {
            write: Mutex::new(write),
            read: Mutex::new(read),
        }
    }
}

impl WebSocketOp for MyClient {
    type Error = MyError;

    async fn send_bin(&self, data: Vec<u8>) -> Result<(), Self::Error> {
        self.write
            .lock()
            .unwrap()
            .send(Message::Binary(data.into()))
            .await
            .unwrap();

        Ok(())
    }

    async fn try_receive_bin(&self) -> Result<Vec<u8>, Self::Error> {
        let Some(Ok(Message::Binary(data))) = self.read.lock().unwrap().next().await else {
            return Err(MyError::Closed);
        };
        return Ok(data.into());
    }
}

pub struct Dsdff(
    Arc<my_core::web_socket::MyClient<MyClient, Atooooooooooo, RandomNumberS, NativeRuntime>>,
);

impl Dsdff {
    pub async fn new() -> Self {
        Self(my_core::web_socket::MyClient::new(MyClient::new().await))
    }
}

impl my_core::traits::BackendRouts for Dsdff {
    type Error = MyError;

    async fn sign_up(&self, input: sign_up::Input) -> Result<sign_up::Result, Self::Error> {
        let result = self
            .0
            .send_and_receive::<sign_up::Input, Result<sign_up::Result, ()>>(
                sign_up::PATH.to_string(),
                input,
                2,
            )
            .await;

        // let Ok(result) = NativeRuntime::timeout_local(Duration::from_secs(2), result).await else {
        //     return Err(MyError::Timeout);
        // };

        match result {
            Ok(o) => match o {
                Ok(o) => Ok(o),
                Err(_) => Err(MyError::ServerError),
            },
            Err(e) => Err(e),
        }
    }

    async fn sign_in(&self, input: sign_in::Input) -> Result<sign_in::Result, Self::Error> {
        let result = self
            .0
            .send_and_receive::<sign_in::Input, Result<sign_in::Result, ()>>(
                sign_in::PATH.to_string(),
                input,
                2,
            )
            .await;

        // let Ok(result) = NativeRuntime::timeout_local(Duration::from_secs(2), result).await else {
        //     return Err(MyError::Timeout);
        // };

        match result {
            Ok(o) => match o {
                Ok(o) => Ok(o),
                Err(_) => Err(MyError::ServerError),
            },
            Err(e) => Err(e),
        }
    }
}
