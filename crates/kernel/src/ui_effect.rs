use crate::types::HashimError;
use infrastructure::actors::Mpsc;
use infrastructure::actors::MpscReceiver;
use infrastructure::actors::MpscSender;
use infrastructure::actors::MultiProducerSingleConsumer;
use infrastructure::actors::Receiver;
use infrastructure::actors::Sender;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use utility::cache::CacheStruct;
use utility::process_manager::MessageToProcessManager;
use utility_ui::domain::HashimSignal;

pub trait Model: 'static {}

pub trait Message {
    type Mdl: Model;
    fn update(
        self: Arc<Self>,
        model: Arc<Self::Mdl>,
        cache: CacheStruct,
        sender_to_process_manager: MpscSender<MessageToProcessManager>,
        aborters: Aborters,
    );
}

type MessageType<Mdl> = Arc<dyn Message<Mdl = Mdl>>;

pub struct Commander<Mdl: Model> {
    sender: MpscSender<MessageType<Mdl>>,
}

impl<Mdl: Model> Clone for Commander<Mdl> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<Mdl: Model> Commander<Mdl> {
    pub(crate) fn new(
        sender_to_process_manager: MpscSender<MessageToProcessManager>,
        model: Arc<Mdl>,
        cache: CacheStruct,
    ) -> Self {
        let (sender_to_commander, receiver_to_commander) = Mpsc::channel();

        Self::commander_actor(receiver_to_commander, sender_to_process_manager, model, cache);

        Self {
            sender: sender_to_commander,
        }
    }

    pub fn send(&self, msg: MessageType<Mdl>) {
        let mut sender = self.sender.clone();
        Rt::spawn_local(async move {
            sender.send(msg).await.unwrap();
        });
    }

    fn commander_actor(
        mut receiver: MpscReceiver<MessageType<Mdl>>,
        sender_to_process_manager: MpscSender<MessageToProcessManager>,
        model: Arc<Mdl>,
        cache: CacheStruct,
    ) {
        Rt::spawn_local(async move {
            let aborters = Aborters::default();

            loop {
                let message = receiver.recv().await.unwrap();

                let model = model.clone();
                let cache = cache.clone();
                let sender_to_process_manager = sender_to_process_manager.clone();
                let aborters = aborters.clone();

                Rt::spawn_local(async move {
                    message.update(model, cache, sender_to_process_manager, aborters);
                });
            }
        });
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct PageId(i32);

pub struct Aborter(Box<dyn FnOnce()>);

#[derive(Clone, Default)]
pub struct Aborters(Arc<Mutex<HashMap<PageId, Aborter>>>);

impl Aborters {
    pub fn register(&self, page: PageId, aborter: Aborter) {
        let mut mutex_guard = self.0.lock().unwrap();
        mutex_guard.insert(page, aborter);
    }

    pub fn abort(&self, page: PageId) {
        let mut mutex_guard = self.0.lock().unwrap();
        if let Some(a) = mutex_guard.remove(&page) {
            a.0();
        }
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
