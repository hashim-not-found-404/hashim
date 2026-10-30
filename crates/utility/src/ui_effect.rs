use crate::cache::CacheStruct;
use crate::handle_errors::handle_error;
use anyhow::Error;
use anyhow::Result;
use dyn_clone::DynClone;
use dyn_clone::clone_trait_object;
use infrastructure::actors::Mpsc;
use infrastructure::actors::MpscReceiver;
use infrastructure::actors::MpscSender;
use infrastructure::actors::MultiProducerSingleConsumer;
use infrastructure::actors::Receiver;
use infrastructure::actors::Sender;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use std::any::Any;
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::Arc;
use tracing::info;

pub trait SigNaaaaaaaaaaaaaaaaal<T>: 'static {
    fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R;
}

pub trait Model: 'static {}

pub trait MessageTrait: Any + Debug + 'static + Send + DynClone {
    // here it should have method that return serializable dyn trait for the intent to store it
}

clone_trait_object!(MessageTrait);

pub type Replier =
    Arc<dyn Fn(Box<dyn MessageTrait>) -> Pin<Box<dyn Future<Output = Result<()>>>> + Send + Sync>;

pub fn new_replier<F, Fut>(a: F) -> Replier
where
    F: Fn(Box<dyn MessageTrait>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + 'static,
{
    Arc::new(
        move |msg: Box<dyn MessageTrait>| -> Pin<Box<dyn Future<Output = Result<()>>>> {
            Box::pin(a(msg))
        },
    )
}

pub struct MessageToCommander {
    pub replier: Replier,
    pub inner: Box<dyn MessageTrait>,
}

impl Debug for MessageToCommander {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageToCommander")
            .field("inner", &self.inner)
            .finish()
    }
}

#[derive(Clone)]
pub struct UiContext {
    pub cache: CacheStruct,
}

pub trait UpdaterTrait: Debug {
    type Mdl: Model;

    fn update(
        &self,
        model: &Self::Mdl,
    ) -> (
        Vec<Box<dyn ApplierTrait<Mdl = Self::Mdl>>>,
        Vec<Box<dyn EffectorTrait>>,
    );
}

pub trait ApplierTrait: Debug {
    type Mdl: Model;

    fn apply(&self, model: &mut Self::Mdl);
}

pub trait EffectorTrait: Debug {
    fn effect(
        &self,
        replier: Replier,
        context: UiContext,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>>;
}

pub trait CastMessageToReducer {
    type Mdl: Model;

    fn cast_message_to_reducer(
        v: Box<dyn MessageTrait>,
    ) -> Result<Box<dyn UpdaterTrait<Mdl = Self::Mdl>>>;
}

#[derive(Clone)]
pub struct Commander {
    sender: MpscSender<MessageToCommander>,
}

impl Commander {
    pub fn new<Mdl, CasMsg>(
        sender_to_error: MpscSender<Error>,
        model: impl SigNaaaaaaaaaaaaaaaaal<Mdl>,
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
            model,
            cache,
        );

        commander
    }

    fn replier(self) -> Replier {
        new_replier(move |msg| {
            let commander = self.clone();
            async move {
                let next = commander.clone().replier();
                commander.async_send_box(next, msg).await
            }
        })
    }

    pub fn sender_with_replier(&self, msg: impl MessageTrait) {
        self.send(self.clone().replier(), msg);
    }

    pub fn send(&self, replier: Replier, msg: impl MessageTrait) {
        let result = self.sender.clone().unbounded_send(MessageToCommander {
            replier,
            inner: Box::new(msg),
        });

        if let Err(e) = result {
            eprintln!("the error from the sender to commander : {e}")
        }
    }

    pub async fn async_send_box(&self, replier: Replier, msg: Box<dyn MessageTrait>) -> Result<()> {
        let mut sender = self.sender.clone();
        sender
            .send(MessageToCommander {
                replier,
                inner: msg,
            })
            .await?;

        Ok(())
    }

    fn commander_actor<Mdl, CasMsg>(
        self,
        mut sender_to_error: MpscSender<Error>,
        mut receiver: MpscReceiver<MessageToCommander>,
        model: impl SigNaaaaaaaaaaaaaaaaal<Mdl>,
        cache: CacheStruct,
    ) where
        Mdl: Model,
        CasMsg: CastMessageToReducer<Mdl = Mdl>,
    {
        Rt::spawn_local(async move {
            let context = UiContext { cache };

            handle_error::<(), _>(sender_to_error.clone(), async || {
                loop {
                    let message = receiver.recv().await?;

                    info!("###########################################################################");
                    info!(?message);

                    let msg = CasMsg::cast_message_to_reducer(message.inner)?;

                    let effect=model.with_mut(|model|{
                        let (change, effect) = msg.update(&model);

                        info!(?change);
                        info!(?effect);

                        for i in change {
                            i.apply(model);
                        }

                        effect

                    });


                    for i in effect {
                        let result = i.effect(message.replier.clone(),context.clone()).await;
                        if let Err(err) = result {
                            let _ = sender_to_error.send(err).await;
                        }
                    }
                }
            })
            .await;
        });
    }
}
