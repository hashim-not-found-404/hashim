use crate::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
#[cfg(not(target_arch = "wasm32"))]
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[cfg(not(target_arch = "wasm32"))]
pub struct MyClient {
    write: Mutex<SplitSink<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, Message>>,
    read: Mutex<SplitStream<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl WebSocketOp for MyClient {
    async fn connect(url: &str) -> Result<Self, DynamicError> {
        let (ws_stream, _) = connect_async(url).await?;

        let (write, read) = ws_stream.split();

        Ok(Self {
            write: Mutex::new(write),
            read: Mutex::new(read),
        })
    }

    async fn send_bin(&self, data: Vec<u8>) -> Result<(), DynamicError> {
        self.write
            .lock()
            .unwrap()
            .send(Message::Binary(data.into()))
            .await?;

        Ok(())
    }

    async fn try_receive_bin(&self) -> Result<Vec<u8>, DynamicError> {
        let mut guard = self.read.lock().unwrap();

        match guard.next().await {
            Some(Ok(message)) => match message {
                Message::Text(_) => Err("it's text".into()),
                Message::Binary(bytes) => Ok(bytes.to_vec()),
                Message::Close(_) => Err("it's closed".into()),
                _ => Err("other message type".into()),
            },
            Some(Err(e)) => Err(e.to_string().into()),
            None => Err("it's closed".into()),
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub struct MyClient {
    write: Mutex<SplitSink<WebSocket, Message>>,
    read: Mutex<SplitStream<WebSocket>>,
}

#[cfg(target_arch = "wasm32")]
impl WebSocketOp for MyClient {
    async fn connect(url: &str) -> Result<Self, DynamicError> {
        let ws = WebSocket::open(url)?;

        // Split into write and read halves
        let (write, read) = ws.split();

        Ok(Self {
            write: Mutex::new(write),
            read: Mutex::new(read),
        })
    }

    async fn send_bin(&self, data: Vec<u8>) -> Result<(), DynamicError> {
        self.write
            .lock()
            .unwrap()
            .send(Message::Bytes(data.into()))
            .await?;

        Ok(())
    }

    async fn try_receive_bin(&self) -> Result<Vec<u8>, DynamicError> {
        let mut guard = self.read.lock().unwrap();

        match guard.next().await {
            Some(Ok(message)) => match message {
                Message::Text(_) => Err("it's text".into()),
                Message::Bytes(bytes) => Ok(bytes.to_vec()),
            },
            Some(Err(e)) => Err(e.into()),
            None => Err("it's closed".into()),
        }
    }
}
