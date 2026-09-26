use crate::cache::CacheStruct;
use crate::handle_errors::handle_error;
use crate::process_manager::MessageToProcessManager;
use anyhow::Error;
use anyhow::Result;
use dyn_clone::DynClone;
use infrastructure::actors::Mpsc;
use infrastructure::actors::MpscReceiver;
use infrastructure::actors::MpscSender;
use infrastructure::actors::MultiProducerSingleConsumer;
use infrastructure::actors::Receiver;
use infrastructure::actors::Sender;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;

pub trait Model: 'static {}

pub trait MessageTrait: Any + Debug + 'static + Send + DynClone {}

pub struct UiContext<Mdl: Model> {
    pub model: Arc<Mdl>,
    pub cache: CacheStruct,
    pub sender_to_process_manager: MpscSender<MessageToProcessManager>,
    pub aborters: Aborters,
    pub sender_to_error: MpscSender<Error>,
}

impl<Mdl: Model> Clone for UiContext<Mdl> {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            cache: self.cache.clone(),
            sender_to_process_manager: self.sender_to_process_manager.clone(),
            aborters: self.aborters.clone(),
            sender_to_error: self.sender_to_error.clone(),
        }
    }
}

pub trait UpdaterTrait {
    type Mdl: Model;

    fn update(
        self: Box<Self>,
        context: UiContext<Self::Mdl>,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>>;
}

pub trait CastMessageToUpdater {
    type Mdl: Model;

    fn cast_message_to_updater(
        v: Box<dyn MessageTrait>,
    ) -> Result<Box<dyn UpdaterTrait<Mdl = Self::Mdl>>>;
}

#[derive(Clone)]
pub struct Commander {
    sender: MpscSender<Box<dyn MessageTrait>>,
}

impl Commander {
    pub fn new<Mdl, CasMsg>(
        sender_to_error: MpscSender<Error>,
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
            sender_to_error,
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
            let _ = sender.send(Box::new(msg)).await;
        });
    }

    fn commander_actor<Mdl, CasMsg>(
        sender_to_error: MpscSender<Error>,
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

            let context = UiContext {
                model,
                cache,
                sender_to_process_manager,
                aborters,
                sender_to_error: sender_to_error.clone(),
            };

            handle_error::<(), _>(sender_to_error.clone(), async || {
                loop {
                    let message = receiver.recv().await?;
                    let message = CasMsg::cast_message_to_updater(message)?;
                    let context = context.clone();
                    let mut sender_to_error = sender_to_error.clone();

                    Rt::spawn_local(async move {
                        let result = message.update(context).await;

                        if let Err(err) = result {
                            let _ = sender_to_error.send(err).await;
                        }
                    });
                }
            })
            .await;
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
