use crate::client::Cache;
use crate::types::ADDRESS;
use crate::types::HashimError;
use infrastructure::actors::Mpsc;
use infrastructure::actors::MpscReceiver;
use infrastructure::actors::MpscSender;
use infrastructure::actors::MultiProducerSingleConsumer;
use infrastructure::actors::Receiver;
use infrastructure::actors::Sender;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use std::sync::Arc;
use std::sync::RwLock;
use utility::cache::CacheStruct;
use utility::cache::CacheUtility;
use utility::cache::EncodeDecodeForRequestAndResponse;
use utility::cache::MessageToCache;
use utility::network::Network;
use utility::network::network_actor;
use utility::process_manager::process_manager_actor;
use utility::types::ReadAndSet;
use utility::ui_effect::Commander;
use utility::ui_effect::Model;
use utility_ui::domain::HashimSignal;

pub fn new<
    Ch: Cache + CacheUtility + 'static,
    Edrr: EncodeDecodeForRequestAndResponse<CacheUtility = Ch>,
    Mdl: Model,
>(
    model: Arc<Mdl>,
) -> Commander<Mdl, Ch> {
    let (sender_to_network, receiver_to_network) = Mpsc::channel();
    let (sender_to_cache, receiver_to_cache) = Mpsc::channel();
    let (sender_to_error, receiver_to_error) = Mpsc::channel();

    let is_online = Arc::new(RwLock::new(false));

    network_actor::<_>(
        receiver_to_network,
        sender_to_error.clone(),
        MyNetwork {
            sender_to_cache: sender_to_cache.clone(),
            is_online:       is_online.clone(),
        },
        format!("ws://{}/ws", ADDRESS),
    );

    let cache = CacheStruct::new::<Edrr>(
        receiver_to_cache,
        sender_to_cache,
        sender_to_network,
        sender_to_error,
        is_online,
    );

    let sender_to_process_manager = process_manager_actor();

    Commander::new(sender_to_process_manager, model, cache)
}

struct MyNetwork<Cu: CacheUtility> {
    sender_to_cache: MpscSender<MessageToCache<Cu>>,
    is_online:       Arc<RwLock<bool>>,
}

impl<Cu: CacheUtility> Network for MyNetwork<Cu> {
    async fn network_state(&mut self, is_online: bool) {
        self.is_online.put(is_online);

        if is_online {
            self.sender_to_cache.send(MessageToCache::WeAreBackOnline).await.unwrap();
        }
    }

    async fn network_sender(&mut self, data: Vec<u8>) {
        self.sender_to_cache.send(MessageToCache::DataFromServer(data)).await.unwrap();
    }
}

fn listen_to_error_actor(
    mut receiver_to_error: MpscReceiver<HashimError>,
    external_errors_signal: impl HashimSignal<String>,
) {
    Rt::spawn_local(async move {
        loop {
            let err = receiver_to_error.recv().await.unwrap();
            external_errors_signal.set(err.to_string());
        }
    });
}
