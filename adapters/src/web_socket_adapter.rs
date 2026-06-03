#[cfg(not(target_arch = "wasm32"))]
pub mod m {
    use crate::prelude::*;
    use futures::stream::{SplitSink, SplitStream};
    use futures::{SinkExt, StreamExt};
    use std::sync::Mutex;
    use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    pub struct S {
        write: Mutex<SplitSink<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, Message>>,
        read: Mutex<SplitStream<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>>,
    }

    impl WebSocketOp for S {
        async fn connect(url: &str) -> Result<Self, DynamicError> {
            let (ws_stream, _) = connect_async(url).await.log()?;
            let (write, read) = ws_stream.split();

            Ok(Self {
                write: Mutex::new(write),
                read: Mutex::new(read),
            })
        }

        async fn send_bin(&self, data: &Vec<u8>) -> Result<(), DynamicError> {
            self.write
                .lock()
                .unwrap()
                .send(Message::Binary(data.clone().into()))
                .await
                .log()?;

            Ok(())
        }

        async fn receive_bin(&self) -> Result<Vec<u8>, DynamicError> {
            let mut guard = self.read.lock().unwrap();

            match guard.next().await {
                Some(Ok(message)) => match message {
                    Message::Text(_) => Err("it's text".into()),
                    Message::Binary(bytes) => Ok(bytes.to_vec()),
                    Message::Close(_) => Err(HashimError::ConnectionClosed.into()),
                    _ => Err("other message type".into()),
                },
                Some(Err(e)) => Err(e.to_string().into()),
                None => Err(HashimError::ConnectionClosed.into()),
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub mod m {
    use crate::prelude::*;
    use futures_util::{
        SinkExt, StreamExt,
        stream::{SplitSink, SplitStream},
    };
    use gloo_net::websocket::{Message, futures::WebSocket};
    use gloo_timers::future::TimeoutFuture;
    use std::sync::Mutex;
    use wasm_bindgen_futures::spawn_local;

    pub struct S {
        write: Mutex<SplitSink<WebSocket, Message>>,
        read: Mutex<SplitStream<WebSocket>>,
    }

    impl WebSocketOp for S {
        async fn connect(url: &str) -> Result<Self, DynamicError> {
            let ws = WebSocket::open(url).log()?;

            // Split into write and read halves
            let (write, read) = ws.split();

            Ok(Self {
                write: Mutex::new(write),
                read: Mutex::new(read),
            })
        }

        async fn send_bin(&self, data: &Vec<u8>) -> Result<(), DynamicError> {
            self.write
                .lock()
                .unwrap()
                .send(Message::Bytes(data.clone().into()))
                .await
                .log()?;

            Ok(())
        }

        async fn receive_bin(&self) -> Result<Vec<u8>, DynamicError> {
            let mut guard = self.read.lock().unwrap();

            match guard.next().await {
                Some(Ok(message)) => match message {
                    Message::Text(_) => Err("it's text".into()),
                    Message::Bytes(bytes) => Ok(bytes.to_vec()),
                },
                Some(Err(e)) => Err(e.into()),
                None => Err(HashimError::ConnectionClosed.into()),
            }
        }
    }
}
