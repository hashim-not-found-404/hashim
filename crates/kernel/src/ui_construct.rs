use crate::client::Cache;
use crate::types::ADDRESS;
use crate::types::HashimError;
use anyhow::Result;
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
use utility::cache::MessageFromServer;
use utility::cache::MessageToCache;
use utility::cache::OpErr;
use utility::cache::OpInput;
use utility::cache::TxnNumber;
use utility::network::Network;
use utility::network::network_actor;
use utility::process_manager::process_manager_actor;
use utility::types::ReadAndSet;
use utility::ui_effect::Commander;
use utility::ui_effect::Model;
use utility_ui::domain::HashimSignal;

pub fn new<Ch: Cache + 'static, Mdl: Model>(model: Arc<Mdl>) -> Commander<Mdl, MyCacheUtility<Ch>> {
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

    let cache = CacheStruct::new(
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

pub struct MyCacheUtility<Ch: Cache> {
    cache: Ch,
}

impl<Ch: Cache> CacheUtility for MyCacheUtility<Ch> {
    async fn new_cache() -> Self {
        Self {
            cache: Ch::new().await,
        }
    }

    async fn get_all_pending_txn(&mut self) -> Vec<(TxnNumber, OpInput<Self>)> {
        let a = self.cache.get_all_txn_input().await.iter().map(|a| (a.txn_number, a.operation));
    }

    async fn clear_state_pending_txn(&mut self) {
        todo!()
    }

    async fn start_state_pending_txn(&mut self) {
        todo!()
    }

    async fn delete_successful_txn_input(&mut self, txn_number: TxnNumber) {
        todo!()
    }

    async fn mark_txn_input_as_faild(&mut self, txn_number: TxnNumber) {
        todo!()
    }

    async fn write_input_to_cache(&mut self, txn_number: TxnNumber, input: OpInput<Self>) {
        todo!()
    }

    async fn write_error_to_cache(&mut self, txn_number: TxnNumber, input: OpErr) {
        todo!()
    }

    async fn encode_the_inputs(&mut self, inputs: Vec<(TxnNumber, OpInput<Self>)>) -> Vec<u8> {
        todo!()
    }

    fn decode_the_response(resp: Vec<u8>) -> Result<MessageFromServer<Self>> {
        todo!()
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
