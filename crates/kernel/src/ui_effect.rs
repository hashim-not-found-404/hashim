use std::sync::Arc;

use crate::client::Cache;
use crate::types::HashimError;
use infrastructure::actors::Mpsc;
use infrastructure::actors::MpscReceiver;
use infrastructure::actors::MpscSender;
use infrastructure::actors::MultiProducerSingleConsumer;
use infrastructure::actors::Receiver;
use infrastructure::actors::Sender;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use utility::cache::CacheStruct;
use utility::process_manager::MessageToProcessManager;
use utility_ui::domain::HashimSignal;

trait Model: 'static {}

trait Message {
    type Mdl: Model;
    fn update(self: Arc<Self>, model: Self::Mdl);
}

type MessageType<Mdl: Model> = Arc<dyn Message<Mdl = Mdl> + 'static>;

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
    pub(crate) fn new<Ch: Cache + 'static>(
        receiver_to_error: MpscReceiver<HashimError>,
        sender_to_process_manager: MpscSender<MessageToProcessManager<Di>>,
        model: Mdl,
        cache: CacheStruct,
    ) -> Self {
        let (sender_to_commander, receiver_to_commander) = Mpsc::channel();

        // listen_to_error_actor(receiver_to_error, &model.external_errors);

        Self::commander_actor::<Ch>(receiver_to_commander, sender_to_process_manager, model, cache);

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

    fn commander_actor<Ch: Cache + 'static>(
        mut receiver: MpscReceiver<MessageType<Mdl>>,
        sender_to_process_manager: MpscSender<MessageToProcessManager<Di>>,
        model: Mdl,
        cache: CacheStruct,
    ) {
        Rt::spawn_local(async move {
            // let commander_local_state = commander::new(sender_to_process_manager);

            loop {
                let message = receiver.recv().await.unwrap();

                let cache = cache.clone();
                // let commander_local_state = commander_local_state.clone();

                Rt::spawn_local(async move {
                    message.update(model);
                });
            }
        });
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
