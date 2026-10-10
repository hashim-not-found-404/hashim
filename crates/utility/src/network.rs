use crate::handle_errors::handle_error;
use anyhow::Error;
use anyhow::Result;
use infrastructure::actors::MpscReceiver;
use infrastructure::actors::MpscSender;
use infrastructure::actors::Receiver;
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

async fn connect<Nw: Network>(network_utils: &mut Nw, url: impl AsRef<str>) -> Result<Ws> {
    network_utils.network_state(false).await?;
    loop {
        if let Ok(ok) = Ws::connect(url.as_ref()).await {
            network_utils.network_state(true).await?;
            return Ok(ok);
        }
        Rt::sleep(Nw::SLEEP_DURATION).await;
    }
}

pub fn network_actor<Nw: Network + 'static>(
    mut receiver_to_network: MpscReceiver<Vec<u8>>,
    sender_to_error: MpscSender<Error>,
    mut network_utils: Nw,
    url: impl AsRef<str> + 'static,
) {
    Rt::spawn_local(async move {
        handle_error::<(), _>(sender_to_error.clone(), async || {
            let mut ws: Ws = connect::<Nw>(&mut network_utils, &url).await?;

            loop {
                let either = Rt::select(receiver_to_network.recv(), ws.receive_bin()).await;

                match either {
                    Either::One(data) => {
                        ws.send_bin(&data?).await?;
                    }
                    Either::Two(data) => {
                        network_utils.network_sender(data?).await?;
                    }
                }
            }
        })
        .await;
    });
}
