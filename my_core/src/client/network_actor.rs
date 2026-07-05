use crate::{
    accounting_domain::db_types,
    client::client_traits::{AllClientTypes, WSClient},
    utility::shared_traits::{Either, MultiProducerSingleConsumer, Receiver, Runtime, Sender},
    utility::utils::{self, ReadAndSet},
};
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

pub(crate) trait Network {
    async fn from_network_status(&mut self, are_we_online: bool);
    async fn sender_to_network(&mut self, data: Vec<u8>);
}

pub(crate) type MessageToNetwork = Vec<u8>;

async fn network_radar<At: AllClientTypes>(
    ws: &Option<At::Ws>,
) -> Result<Vec<u8>, utils::DynamicError> {
    match &ws {
        Some(ws) => ws.receive_bin().await,
        None => Err(db_types::HashimError::ConnectionClosed.into()),
    }
}

async fn connect<At: AllClientTypes, Nw: Network>(
    is_online: Arc<RwLock<bool>>,
    sender_to_cache: &mut Nw,
    url: &String,
    ws: &mut Option<At::Ws>,
) {
    is_online.set(false);

    if let Ok(ok) = At::Ws::connect(url.as_str()).await {
        *ws = Some(ok);

        sender_to_cache.from_network_status(true).await;

        is_online.set(true);

        return;
    }
    At::Rt::sleep(Duration::from_secs(5)).await;
}

pub(crate) fn network_actor<At: AllClientTypes, Nw: Network + 'static>(
    mut receiver_to_network: <At::Mpsc as MultiProducerSingleConsumer>::Receiver<MessageToNetwork>,
    mut sender_to_cache: Nw,
    mut sender_to_error: <At::Mpsc as MultiProducerSingleConsumer>::Sender<db_types::HashimError>,
    is_online: Arc<RwLock<bool>>,
    url: String,
) {
    At::Rt::spawn_local(async move {
        let mut ws: Option<At::Ws> = None;

        loop {
            match At::Rt::select(receiver_to_network.recv(), network_radar::<At>(&ws)).await {
                Either::One(r) => match r.unwrap() {
                    data => match &ws {
                        Some(ws1) => {
                            let result = ws1.send_bin(&data).await;
                            if result.is_err() {
                                connect::<At, Nw>(
                                    is_online.clone(),
                                    &mut sender_to_cache,
                                    &url,
                                    &mut ws,
                                )
                                .await;
                            }
                        }
                        None => At::Rt::sleep(Duration::from_secs(5)).await,
                    },
                },
                Either::Two(from_network) => match from_network {
                    Ok(data) => {
                        sender_to_cache.sender_to_network(data).await;
                    }
                    Err(_) => {
                        sender_to_error
                            .send(db_types::HashimError::ConnectionClosed)
                            .await
                            .unwrap();
                        connect::<At, Nw>(is_online.clone(), &mut sender_to_cache, &url, &mut ws)
                            .await;
                    }
                },
            }
        }
    });
}
