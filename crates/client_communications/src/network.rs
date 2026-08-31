use adapters::traits::Either;
use adapters::traits::Runtime;
use std::time::Duration;
use utility::types::DynamicError;

const SLEEP_DURATION: Duration = Duration::from_millis(100);

pub trait WSClient: Sized {
    fn connect(url: &str) -> impl Future<Output = Result<Self, DynamicError>>;
    fn send_bin(&mut self, data: &[u8]) -> impl Future<Output = Result<(), DynamicError>>;
    fn receive_bin(&mut self) -> impl Future<Output = Result<Vec<u8>, DynamicError>>;
}

pub(crate) trait Network {
    async fn network_state(&mut self, is_online: bool);
    async fn network_sender(&mut self, data: Vec<u8>);
    async fn network_reciever(&mut self) -> Vec<u8>;
    async fn send_error(&mut self, error: DynamicError);
}

async fn network_radar<Ws: WSClient>(ws: Option<&mut Ws>) -> Result<Vec<u8>, DynamicError> {
    match ws {
        Some(ws) => ws.receive_bin().await,
        None => Err("error".into()),
    }
}

async fn connect<Rt: Runtime, Ws: WSClient, Nw: Network>(
    network_utils: &mut Nw,
    url: &str,
    ws: &mut Option<Ws>,
) {
    network_utils.network_state(false).await;

    if let Ok(ok) = Ws::connect(url).await {
        *ws = Some(ok);
        network_utils.network_state(true).await;
        return;
    }
    Rt::sleep(SLEEP_DURATION).await;
}

pub(crate) fn network_actor<Rt: Runtime, Ws: WSClient, Nw: Network + 'static>(
    mut network_utils: Nw,
    url: String,
) {
    Rt::spawn_local(async move {
        let mut ws: Option<Ws> = None;

        loop {
            match Rt::select(network_utils.network_reciever(), network_radar::<Ws>(ws.as_mut()))
                .await
            {
                Either::One(data) => {
                    match &mut ws {
                        Some(ws1) => {
                            let result = ws1.send_bin(&data).await;
                            if result.is_err() {
                                connect::<Rt, Ws, Nw>(&mut network_utils, &url, &mut ws).await;
                            }
                        }
                        None => Rt::sleep(SLEEP_DURATION).await,
                    }
                }

                Either::Two(from_network) => {
                    match from_network {
                        Ok(data) => {
                            network_utils.network_sender(data).await;
                        }
                        Err(error) => {
                            network_utils.send_error(error).await;
                            connect::<Rt, Ws, Nw>(&mut network_utils, &url, &mut ws).await;
                        }
                    }
                }
            }
        }
    });
}

#[cfg(feature = "infrastructure")]
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

#[cfg(feature = "infrastructure")]
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
