use crate::cache::CacheStruct;
use crate::process_manager::MessageToProcessManager;
use infrastructure::actors::Mpsc;
use infrastructure::actors::MpscReceiver;
use infrastructure::actors::MpscSender;
use infrastructure::actors::MultiProducerSingleConsumer;
use infrastructure::actors::Receiver;
use infrastructure::actors::Sender;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::Mutex;

pub trait Model: 'static {}

pub trait Message: Debug {
    type Mdl: Model;
    type Cache;

    fn update(
        self: Arc<Self>,
        model: Arc<Self::Mdl>,
        cache: CacheStruct<Self::Cache>,
        sender_to_process_manager: MpscSender<MessageToProcessManager>,
        aborters: Aborters,
    );
}

type MessageType<Mdl, Ch> = Arc<dyn Message<Mdl = Mdl, Cache = Ch>>;

pub struct Commander<Mdl: Model, Ch> {
    sender: MpscSender<MessageType<Mdl, Ch>>,
}

impl<Mdl: Model, Ch> Clone for Commander<Mdl, Ch> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<Mdl, Ch> Commander<Mdl, Ch>
where
    Mdl: Model,
    Ch: 'static,
{
    pub fn new(
        sender_to_process_manager: MpscSender<MessageToProcessManager>,
        model: Arc<Mdl>,
        cache: CacheStruct<Ch>,
    ) -> Self {
        let (sender_to_commander, receiver_to_commander) = Mpsc::channel();

        Self::commander_actor(receiver_to_commander, sender_to_process_manager, model, cache);

        Self {
            sender: sender_to_commander,
        }
    }

    pub fn send<Msg>(&self, msg: Msg)
    where
        Msg: Message<Mdl = Mdl, Cache = Ch> + 'static,
    {
        let mut sender = self.sender.clone();
        Rt::spawn_local(async move {
            sender.send(Arc::new(msg)).await.unwrap();
        });
    }

    fn commander_actor(
        mut receiver: MpscReceiver<MessageType<Mdl, Ch>>,
        sender_to_process_manager: MpscSender<MessageToProcessManager>,
        model: Arc<Mdl>,
        cache: CacheStruct<Ch>,
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
