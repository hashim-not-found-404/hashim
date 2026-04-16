use std::{collections::HashMap, sync::RwLock};

use futures_util::{SinkExt, StreamExt};
use my_core::request_response::sign_up::Input;
use serde_json;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, connect_async, tungstenite::Message};

pub struct MyClient {
    ws_stream: RwLock<tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>>,
    pool_of_response: HashMap<u64, my_core::request_response::business_layer::Paths>,
}

impl MyClient {
    async fn new() -> Self {
        let (ws_stream, _) =
            connect_async(format!("ws://{}/ws", my_core::request_response::ADDRESS))
                .await
                .unwrap();
        Self {
            ws_stream: RwLock::new(ws_stream),
            pool_of_response: HashMap::default(),
        }
    }

    async fn send(&self, input: my_core::request_response::business_layer::Input) {
        let input = serde_json::to_string(&input).unwrap();
        self.ws_stream
            .write()
            .unwrap()
            .send(Message::Text(input.into()))
            .await
            .unwrap();
    }

    async fn receive(&self) {
        if let Some(reply) = self.ws_stream.write().unwrap().next().await {
            let reply = reply.unwrap();
            let reply: my_core::request_response::business_layer::Result =
                serde_json::from_str(&reply.to_text().unwrap()).unwrap();
            self.pool_of_response.insert(reply, v)
        }
    }
}

// i need WAMP

impl my_core::traits::BackendRouts for MyClient {
    type Error = ();

    async fn sign_up(
        &self,
        input: my_core::request_response::business_layer::Input,
    ) -> my_core::request_response::sign_up::Result<Self::Error> {
        self.send(input);
        // let input = serde_json::to_string(&input).unwrap();
        // self.ws_stream
        //     .write()
        //     .unwrap()
        //     .send(Message::Text(input.into()))
        //     .await
        //     .unwrap();
        todo!()
    }
}

#[tokio::main]
async fn main() {
    let (mut ws_stream, _) = connect_async("ws://127.0.0.1:8080/ws").await.unwrap();

    println!("Connected to server!\n");

    ws_stream
        .send(Message::Text("msg".to_string().into()))
        .await
        .unwrap();

    if let Some(Ok(reply)) = ws_stream.next().await {
        println!("📥 Got: {}\n", reply);
    }
}

// here i need to add the impls to make the ws work with the front end
