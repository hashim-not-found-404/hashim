use crate::prelude::*;

pub(crate) type MessageToNetwork = Vec<u8>;

async fn network_radar<At: AllClientTypes>(ws: &Option<At::Ws>) -> Result<Vec<u8>, DynamicError> {
    match &ws {
        Some(ws) => ws.receive_bin().await,
        None => Err(HashimError::ConnectionClosed.into()),
    }
}

async fn connect<At: AllClientTypes>(
    is_online: Arc<RwLock<bool>>,
    sender_to_cache: &mut <At::Mpsc as MultiProducerSingleConsumer>::Sender<
        cache_actor::MessageToCache<At>,
    >,
    url: &String,
    ws: &mut Option<At::Ws>,
) {
    is_online.set(false);

    if let Ok(ok) = At::Ws::connect(url.as_str()).await {
        *ws = Some(ok);

        sender_to_cache
            .send(cache_actor::MessageToCache::WeAreBackOnline)
            .await
            .unwrap();

        is_online.set(true);

        return;
    }
    At::Rt::sleep(Duration::from_secs(5)).await;
}

pub(crate) fn network_actor<At: AllClientTypes>(
    mut receiver_to_network: <At::Mpsc as MultiProducerSingleConsumer>::Receiver<MessageToNetwork>,
    mut sender_to_cache: <At::Mpsc as MultiProducerSingleConsumer>::Sender<
        cache_actor::MessageToCache<At>,
    >,
    mut sender_to_error: <At::Mpsc as MultiProducerSingleConsumer>::Sender<HashimError>,
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
                                connect::<At>(
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
                        sender_to_cache
                            .send(cache_actor::MessageToCache::DataFromServer(data))
                            .await
                            .unwrap();
                    }
                    Err(_) => {
                        sender_to_error
                            .send(HashimError::ConnectionClosed)
                            .await
                            .unwrap();
                        connect::<At>(is_online.clone(), &mut sender_to_cache, &url, &mut ws).await;
                    }
                },
            }
        }
    });
}
