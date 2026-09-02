#[cfg(not(target_arch = "wasm32"))]
pub mod target {
    use futures::SinkExt;
    use futures::StreamExt;
    use futures::stream::SplitSink;
    use futures::stream::SplitStream;
    use my_core::client::network_actor::WSClient;
    use my_core::domain::utility::types::HashimError;
    use my_core::utility::traits::DynamicError;
    use my_core::utility::utils::LogError;
    use tokio::net::TcpStream;
    use tokio_tungstenite::MaybeTlsStream;
    use tokio_tungstenite::WebSocketStream;
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;

    pub struct S {
        write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        read:  SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    }

    impl WSClient for S {
        async fn connect(url: &str) -> Result<Self, DynamicError> {
            let (ws_stream, _) = connect_async(url).await.log()?;
            let (write, read) = ws_stream.split();

            Ok(Self {
                write,
                read,
            })
        }

        async fn send_bin(&mut self, data: &[u8]) -> Result<(), DynamicError> {
            self.write.send(Message::Binary(data.to_vec().into())).await.log()?;

            Ok(())
        }

        async fn receive_bin(&mut self) -> Result<Vec<u8>, DynamicError> {
            match self.read.next().await {
                Some(Ok(message)) => {
                    match message {
                        Message::Text(_) => Err("it's text".into()),
                        Message::Binary(bytes) => Ok(bytes.to_vec()),
                        Message::Close(_) => Err(HashimError::ConnectionClosed.into()),
                        _ => Err("other message type".into()),
                    }
                }
                Some(Err(e)) => Err(e.to_string().into()),
                None => Err(HashimError::ConnectionClosed.into()),
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub mod target {
    use futures_util::SinkExt;
    use futures_util::StreamExt;
    use futures_util::stream::SplitSink;
    use futures_util::stream::SplitStream;
    use gloo_net::websocket::Message;
    use gloo_net::websocket::futures::WebSocket;
    use my_core::client::network_actor::WSClient;
    use my_core::domain::utility::types::HashimError;
    use my_core::utility::traits::DynamicError;
    use my_core::utility::utils::LogError;
    use std::sync::Mutex;

    pub struct S {
        write: SplitSink<WebSocket, Message>,
        read:  SplitStream<WebSocket>,
    }

    impl WSClient for S {
        async fn connect(url: &str) -> Result<Self, DynamicError> {
            let ws = WebSocket::open(url).log()?;
            let (mut write, read) = ws.split();
            write.send(Message::Bytes(Vec::new())).await.log()?;

            Ok(Self {
                write,
                read,
            })
        }

        async fn send_bin(&mut self, data: &[u8]) -> Result<(), DynamicError> {
            self.write.send(Message::Bytes(data.clone().into())).await.log()?;

            Ok(())
        }

        async fn receive_bin(&mut self) -> Result<Vec<u8>, DynamicError> {
            match self.read.next().await {
                Some(Ok(message)) => {
                    match message {
                        Message::Text(_) => Err("it's text".into()),
                        Message::Bytes(bytes) => Ok(bytes.to_vec()),
                    }
                }
                Some(Err(e)) => Err(e.into()),
                None => Err(HashimError::ConnectionClosed.into()),
            }
        }
    }
}
