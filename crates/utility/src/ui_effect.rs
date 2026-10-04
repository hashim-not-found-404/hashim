use crate::cache::CacheStruct;
use crate::handle_errors::handle_error;
use crate::process_manager::MessageToProcessManager;
use crate::process_manager::ProcessId;
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

pub trait MessageTrait: Any + Debug + 'static + Send + DynClone {
    // here it should have method that return serializable dyn trait for the intent to store it
}

pub struct MessageToCommander {
    pub process_id: ProcessId,
    pub inner: Box<dyn MessageTrait>,
}

#[derive(Clone)]
pub struct UiContext {
    pub cache: CacheStruct,
    pub sender_to_process_manager: MpscSender<MessageToProcessManager>,
    pub aborters: Aborters,
    pub sender_to_error: MpscSender<Error>,
    pub sender_to_commander: Commander,
}

pub trait ReducerTrait {
    type Mdl: Model;

    fn reduce(
        &self,
        model: &Self::Mdl,
        process_id: ProcessId,
    ) -> Result<(
        Vec<Box<dyn UpdaterTrait<Mdl = Self::Mdl>>>,
        Vec<Box<dyn EffectorTrait>>,
    )>;
}

pub trait UpdaterTrait {
    type Mdl: Model;

    fn update(&self, model: &Self::Mdl, process_id: ProcessId);
}

pub trait EffectorTrait {
    fn effect(&self, context: UiContext) -> Pin<Box<dyn Future<Output = Result<()>>>>;
}

pub trait CastMessageToReducer {
    type Mdl: Model;

    fn cast_message_to_reducer(
        v: Box<dyn MessageTrait>,
    ) -> Result<Box<dyn ReducerTrait<Mdl = Self::Mdl>>>;
}

#[derive(Clone)]
pub struct Commander {
    sender: MpscSender<MessageToCommander>,
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
        CasMsg: CastMessageToReducer<Mdl = Mdl>,
    {
        let (sender_to_commander, receiver_to_commander) = Mpsc::channel();

        let commander = Self {
            sender: sender_to_commander,
        };

        commander.clone().commander_actor::<Mdl, CasMsg>(
            sender_to_error,
            receiver_to_commander,
            sender_to_process_manager,
            model,
            cache,
        );

        commander
    }

    pub fn send<Msg>(&self, process_id: ProcessId, msg: Msg)
    where
        Msg: MessageTrait,
    {
        let mut sender = self.sender.clone();
        Rt::spawn_local(async move {
            let _ = sender
                .send(MessageToCommander {
                    process_id,
                    inner: Box::new(msg),
                })
                .await;
        });
    }

    fn commander_actor<Mdl, CasMsg>(
        self,
        sender_to_error: MpscSender<Error>,
        mut receiver: MpscReceiver<MessageToCommander>,
        sender_to_process_manager: MpscSender<MessageToProcessManager>,
        model: Arc<Mdl>,
        cache: CacheStruct,
    ) where
        Mdl: Model,
        CasMsg: CastMessageToReducer<Mdl = Mdl>,
    {
        Rt::spawn_local(async move {
            let aborters = Aborters::default();

            let context = UiContext {
                cache,
                sender_to_process_manager,
                aborters,
                sender_to_error: sender_to_error.clone(),
                sender_to_commander: self,
            };

            handle_error::<(), _>(sender_to_error.clone(), async || {
                loop {
                    let message = receiver.recv().await?;
                    let msg = CasMsg::cast_message_to_reducer(message.inner)?;
                    let (change, effect) = msg.reduce(&model, message.process_id)?;

                    for i in change {
                        i.update(&model, message.process_id);
                    }

                    let context = context.clone();
                    let mut sender_to_error = sender_to_error.clone();

                    Rt::spawn_local(async move {
                        for i in effect {
                            let result = i.effect(context.clone()).await;
                            if let Err(err) = result {
                                let _ = sender_to_error.send(err).await;
                            }
                        }
                    });
                }
            })
            .await;
        });
    }
}

pub struct Aborter(Box<dyn FnOnce()>);

impl Aborter {
    pub(crate) fn new(a: impl FnOnce() + 'static) -> Self {
        Self(Box::new(a))
    }
}

#[derive(Clone, Default)]
pub struct Aborters(Arc<Mutex<HashMap<ProcessId, Aborter>>>);

impl Aborters {
    pub fn register(&self, process_id: ProcessId, aborter: Aborter) {
        let mut mutex_guard = self.0.lock().unwrap();
        mutex_guard.insert(process_id, aborter);
    }

    pub fn abort(&self, process_id: ProcessId) {
        let mut mutex_guard = self.0.lock().unwrap();
        if let Some(a) = mutex_guard.remove(&process_id) {
            a.0();
        }
    }
}
