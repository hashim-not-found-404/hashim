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
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;

pub trait Model: 'static {}

pub trait MessageTrait: Debug + 'static {}

pub trait UpdaterTrait<Mdl: Model> {
    type Cache;

    fn update(
        self: Box<Self>,
        model: Arc<Mdl>,
        cache: CacheStruct<Self::Cache>,
        sender_to_process_manager: MpscSender<MessageToProcessManager>,
        aborters: Aborters,
    ) -> Pin<Box<dyn Future<Output = ()>>>;
}

pub trait Caster {
    fn cast_message_to_updater<Mdl, Ch>(
        v: Box<dyn MessageTrait>,
    ) -> Box<dyn UpdaterTrait<Mdl, Cache = Ch>>
    where
        Mdl: Model,
        Ch: 'static;
}

#[derive(Clone)]
pub struct Commander {
    sender: MpscSender<Box<dyn MessageTrait>>,
}

impl Commander {
    pub fn new<Mdl, Ch, Cas>(
        sender_to_process_manager: MpscSender<MessageToProcessManager>,
        model: Arc<Mdl>,
        cache: CacheStruct<Ch>,
    ) -> Self
    where
        Mdl: Model,
        Ch: 'static,
        Cas: Caster,
    {
        let (sender_to_commander, receiver_to_commander) = Mpsc::channel();

        Self::commander_actor::<Mdl, Ch, Cas>(
            receiver_to_commander,
            sender_to_process_manager,
            model,
            cache,
        );

        Self {
            sender: sender_to_commander,
        }
    }

    pub fn send<Msg>(&self, msg: Msg)
    where
        Msg: MessageTrait,
    {
        let mut sender = self.sender.clone();
        Rt::spawn_local(async move {
            sender.send(Box::new(msg)).await.unwrap();
        });
    }

    fn commander_actor<Mdl, Ch, Cas>(
        mut receiver: MpscReceiver<Box<dyn MessageTrait>>,
        sender_to_process_manager: MpscSender<MessageToProcessManager>,
        model: Arc<Mdl>,
        cache: CacheStruct<Ch>,
    ) where
        Mdl: Model,
        Ch: 'static,
        Cas: Caster,
    {
        Rt::spawn_local(async move {
            let aborters = Aborters::default();

            loop {
                let message = receiver.recv().await.unwrap();
                let message = Cas::cast_message_to_updater(message);

                let model = model.clone();
                let cache = cache.clone();
                let sender_to_process_manager = sender_to_process_manager.clone();
                let aborters = aborters.clone();

                Rt::spawn_local(async move {
                    message.update(model, cache, sender_to_process_manager, aborters).await;
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
