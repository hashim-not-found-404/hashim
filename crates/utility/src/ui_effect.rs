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

pub trait MessageTrait: Debug + 'static + Send {}

pub trait UpdaterTrait {
    type Mdl: Model;

    fn update(
        self: Box<Self>,
        model: Arc<Self::Mdl>,
        cache: CacheStruct,
        sender_to_process_manager: MpscSender<MessageToProcessManager>,
        aborters: Aborters,
    ) -> Pin<Box<dyn Future<Output = ()>>>;
}

pub trait CastMessageToUpdater {
    type Mdl: Model;

    fn cast_message_to_updater(v: Box<dyn MessageTrait>) -> Box<dyn UpdaterTrait<Mdl = Self::Mdl>>;
}

#[derive(Clone)]
pub struct Commander {
    sender: MpscSender<Box<dyn MessageTrait>>,
}

impl Commander {
    pub fn new<Mdl, CasMsg>(
        sender_to_process_manager: MpscSender<MessageToProcessManager>,
        model: Arc<Mdl>,
        cache: CacheStruct,
    ) -> Self
    where
        Mdl: Model,
        CasMsg: CastMessageToUpdater<Mdl = Mdl>,
    {
        let (sender_to_commander, receiver_to_commander) = Mpsc::channel();

        Self::commander_actor::<Mdl, CasMsg>(
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

    fn commander_actor<Mdl, CasMsg>(
        mut receiver: MpscReceiver<Box<dyn MessageTrait>>,
        sender_to_process_manager: MpscSender<MessageToProcessManager>,
        model: Arc<Mdl>,
        cache: CacheStruct,
    ) where
        Mdl: Model,
        CasMsg: CastMessageToUpdater<Mdl = Mdl>,
    {
        Rt::spawn_local(async move {
            let aborters = Aborters::default();

            loop {
                let message = receiver.recv().await.unwrap();
                let message = CasMsg::cast_message_to_updater(message);

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
