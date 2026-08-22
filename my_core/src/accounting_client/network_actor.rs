use crate::utility::traits;
use crate::utility::traits::DynamicError;
use crate::utility::traits::Either;
use std::time::Duration;

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

async fn connect<Rt: traits::Runtime, Ws: WSClient, Nw: Network>(
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
    Rt::sleep(Duration::from_secs(5)).await;
}

pub(crate) fn network_actor<Rt: traits::Runtime, Ws: WSClient, Nw: Network + 'static>(
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
                        None => Rt::sleep(Duration::from_secs(5)).await,
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
