use futures::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use gloo_net::websocket::{Message, futures::WebSocket};
use my_core::{
    request_response::{sign_up, transport_layer},
    web_socket::WebSocketOp,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use wasm_bindgen_futures::spawn_local;

#[derive(Debug, Clone)]
pub enum MyError {
    DecodingError,
    Closed,
    ServerError,
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
            Self::OtherUnexpectedStatusCode(s) => {
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

#[derive(Clone)]
pub struct MyClient {
    write: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    read: Arc<Mutex<SplitStream<WebSocket>>>,
}

impl MyClient {
    fn new() -> Self {
        let url = format!("ws://{}/ws", my_core::request_response::ADDRESS);
        let ws = WebSocket::open(url.as_str()).unwrap();

        let (write, read) = ws.split();

        Self {
            write: Arc::new(Mutex::new(write)),
            read: Arc::new(Mutex::new(read)),
        }
    }
}

impl my_core::web_socket::WebSocketOp for MyClient {
    type Error = MyError;

    async fn send_bin(&self, data: Vec<u8>) -> Result<(), Self::Error> {
        self.write
            .lock()
            .unwrap()
            .send(Message::Bytes(data))
            .await
            .unwrap();

        Ok(())
    }

    async fn try_receive_bin(&self) -> Result<Vec<u8>, Self::Error> {
        let Some(Ok(Message::Bytes(data))) = self.read.lock().unwrap().next().await else {
            return Err(MyError::Closed);
        };
        return Ok(data);
    }
}

pub struct Dsdff(my_core::web_socket::MyClient<MyClient, Atooooooooooo>);

impl Default for Dsdff {
    fn default() -> Self {
        let transport = MyClient::new();
        let cl = Dsdff(my_core::web_socket::MyClient::new(transport));
        cl
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
            )
            .await;

        match result {
            Ok(o) => match o {
                Ok(o) => Ok(o),
                Err(_) => Err(MyError::ServerError),
            },
            Err(e) => Err(e),
        }
    }
}

fn just_a_test() {
    let client = MyClient::new();

    let client1 = client.clone();
    spawn_local(async move {
        client1
            .send_bin("hashem".as_bytes().to_vec())
            .await
            .unwrap();
    });

    let client1 = client.clone();
    spawn_local(async move {
        let a = client1.try_receive_bin().await.unwrap();
        dbg!(a);
    })
}

#[cfg(test)]
mod tests {
    use my_core::traits::BackendRouts;

    use super::*;

    #[test]
    fn test_name() {
        let transport = MyClient::new();
        let cl = Dsdff(my_core::web_socket::MyClient::new(transport));
        let input = sign_up::Input {
            name: None,
            user_id: "hashem".into(),
            password: "ato".to_string(),
        };
        cl.sign_up(input);
    }
}

// TODO : complete the server to make it work
// TODO : try tokio instead
