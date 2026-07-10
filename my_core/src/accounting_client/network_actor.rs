use crate::{
    accounting_domain::types,
    utility::{
        traits::{self, Either, MultiProducerSingleConsumer, Receiver, Runtime, Sender},
        utils::{self, ReadAndSet},
    },
};
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

pub trait WSClient: Sized {
    fn connect(url: &str) -> impl Future<Output = Result<Self, utils::DynamicError>>;
    fn send_bin(&self, data: &Vec<u8>) -> impl Future<Output = Result<(), utils::DynamicError>>;
    fn receive_bin(&self) -> impl Future<Output = Result<Vec<u8>, utils::DynamicError>>;
}

pub(crate) trait Network {
    async fn from_network_status(&mut self);
    async fn sender_to_network(&mut self, data: Vec<u8>);
}

pub(crate) type MessageToNetwork = Vec<u8>;

async fn network_radar<
    Rt: traits::Runtime,
    Mpsc: traits::MultiProducerSingleConsumer,
    Ws: WSClient,
>(
    ws: &Option<Ws>,
) -> Result<Vec<u8>, utils::DynamicError> {
    match &ws {
        Some(ws) => ws.receive_bin().await,
        None => Err(types::HashimError::ConnectionClosed.into()),
    }
}

async fn connect<
    Rt: traits::Runtime,
    Mpsc: traits::MultiProducerSingleConsumer,
    Ws: WSClient,
    Nw: Network,
>(
    is_online: Arc<RwLock<bool>>,
    sender_to_cache: &mut Nw,
    url: &String,
    ws: &mut Option<Ws>,
) {
    is_online.put(false);

    if let Ok(ok) = Ws::connect(url.as_str()).await {
        *ws = Some(ok);

        sender_to_cache.from_network_status().await;

        is_online.put(true);

        return;
    }
    Rt::sleep(Duration::from_secs(5)).await;
}

pub(crate) fn network_actor<
    Rt: traits::Runtime,
    Mpsc: traits::MultiProducerSingleConsumer,
    Ws: WSClient,
    Nw: Network + 'static,
>(
    mut receiver_to_network: Mpsc::Receiver<MessageToNetwork>,
    mut sender_to_cache: Nw,
    mut sender_to_error: Mpsc::Sender<types::HashimError>,
    is_online: Arc<RwLock<bool>>,
    url: String,
) {
    Rt::spawn_local(async move {
        let mut ws: Option<Ws> = None;

        loop {
            match Rt::select(
                receiver_to_network.recv(),
                network_radar::<Rt, Mpsc, Ws>(&ws),
            )
            .await
            {
                Either::One(r) => match r.unwrap() {
                    data => match &ws {
                        Some(ws1) => {
                            let result = ws1.send_bin(&data).await;
                            if result.is_err() {
                                connect::<Rt, Mpsc, Ws, Nw>(
                                    is_online.clone(),
                                    &mut sender_to_cache,
                                    &url,
                                    &mut ws,
                                )
                                .await;
                            }
                        }
                        None => Rt::sleep(Duration::from_secs(5)).await,
                    },
                },
                Either::Two(from_network) => match from_network {
                    Ok(data) => {
                        sender_to_cache.sender_to_network(data).await;
                    }
                    Err(_) => {
                        sender_to_error
                            .send(types::HashimError::ConnectionClosed)
                            .await
                            .unwrap();
                        connect::<Rt, Mpsc, Ws, Nw>(
                            is_online.clone(),
                            &mut sender_to_cache,
                            &url,
                            &mut ws,
                        )
                        .await;
                    }
                },
            }
        }
    });
}
