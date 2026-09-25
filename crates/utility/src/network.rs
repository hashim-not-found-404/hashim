use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use infrastructure::actors::MpscReceiver;
use infrastructure::actors::MpscSender;
use infrastructure::actors::Receiver;
use infrastructure::actors::Sender;
use infrastructure::runtime::Either;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use infrastructure::web_socket_adapter::WSClient;
use infrastructure::web_socket_adapter::Ws;
use std::time::Duration;

pub trait Network {
    const SLEEP_DURATION: Duration = Duration::from_millis(100);
    fn network_state(&mut self, is_online: bool) -> impl Future<Output = Result<()>>;
    fn network_sender(&mut self, data: Vec<u8>) -> impl Future<Output = Result<()>>;
}

async fn network_radar(ws: Option<&mut Ws>) -> Result<Vec<u8>> {
    ws.context("context")?.receive_bin().await
}

async fn connect<Nw: Network>(
    network_utils: &mut Nw,
    url: impl AsRef<str>,
    ws: &mut Option<Ws>,
) -> Result<()> {
    network_utils.network_state(false).await?;

    if let Ok(ok) = Ws::connect(url.as_ref()).await {
        *ws = Some(ok);
        network_utils.network_state(true).await?;
        return Ok(());
    }
    Rt::sleep(Nw::SLEEP_DURATION).await;

    Ok(())
}

pub fn network_actor<Nw: Network + 'static>(
    mut receiver_to_network: MpscReceiver<Vec<u8>>,
    mut sender_to_error: MpscSender<Error>,
    mut network_utils: Nw,
    url: impl AsRef<str> + 'static,
) {
    Rt::spawn_local(async move {
        let mut ws: Option<Ws> = None;

        loop {
            if let Err(err) = fun_name(
                &mut receiver_to_network,
                &mut sender_to_error,
                &mut network_utils,
                &url,
                &mut ws,
            )
            .await
            {
                let _ = sender_to_error.send(err).await;
            };
        }
    });
}

async fn fun_name<Nw: Network + 'static>(
    receiver_to_network: &mut MpscReceiver<Vec<u8>>,
    sender_to_error: &mut MpscSender<Error>,
    network_utils: &mut Nw,
    url: impl AsRef<str>,
    ws: &mut Option<Ws>,
) -> Result<()> {
    match Rt::select(receiver_to_network.recv(), network_radar(ws.as_mut())).await {
        Either::One(data) => {
            let data = data?;

            match ws {
                Some(ws1) => {
                    let result = ws1.send_bin(&data).await;
                    if result.is_err() {
                        connect::<Nw>(network_utils, url, ws).await?;
                    }
                }
                None => Rt::sleep(Nw::SLEEP_DURATION).await,
            }
        }

        Either::Two(from_network) => match from_network {
            Ok(data) => {
                network_utils.network_sender(data).await?;
            }
            Err(error) => {
                sender_to_error.send(error).await?;
                connect::<Nw>(network_utils, url, ws).await?;
            }
        },
    }

    Ok(())
}
