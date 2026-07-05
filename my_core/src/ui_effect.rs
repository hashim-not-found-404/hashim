use crate::{
    cache_actor,
    client_traits::{AllClientTypes, HashimSignal},
    db_types, mbg, process_manager,
    shared_traits::{MultiProducerSingleConsumer, Receiver, Runtime, Sender},
    ui_model,
    ui_updaters::{self, Mvu},
};
use std::sync::{Arc, Mutex};

pub struct Commander<At: AllClientTypes> {
    sender: <At::Mpsc as MultiProducerSingleConsumer>::Sender<ui_model::Message>,
}

impl<At: AllClientTypes> Clone for Commander<At> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<At: AllClientTypes> Commander<At> {
    pub(crate) fn new(
        receiver_to_error: <At::Mpsc as MultiProducerSingleConsumer>::Receiver<
            db_types::HashimError,
        >,
        sender_to_process_manager: <At::Mpsc as MultiProducerSingleConsumer>::Sender<
            process_manager::MessageToProcessManager<At>,
        >,
        model: &'static ui_model::Model<At>,
        cache: cache_actor::CacheStruct<At>,
    ) -> Self {
        let (sender_to_commander, receiver_to_commander) = At::Mpsc::channel();

        listen_to_error_actor::<At>(receiver_to_error, &model.external_errors);

        Self::commander_actor(
            receiver_to_commander,
            sender_to_commander.clone(),
            sender_to_process_manager,
            model,
            cache,
        );

        Self {
            sender: sender_to_commander,
        }
    }

    pub fn send(&self, msg: ui_model::Message) {
        let mut sender = self.sender.clone();
        At::Rt::spawn_local(async move {
            sender.send(msg).await.unwrap();
        });
    }

    fn commander_actor(
        mut receiver: <At::Mpsc as MultiProducerSingleConsumer>::Receiver<ui_model::Message>,
        sender_to_commander: <At::Mpsc as MultiProducerSingleConsumer>::Sender<ui_model::Message>,
        sender_to_process_manager: <At::Mpsc as MultiProducerSingleConsumer>::Sender<
            process_manager::MessageToProcessManager<At>,
        >,
        model: &'static ui_model::Model<At>,
        cache: cache_actor::CacheStruct<At>,
    ) {
        At::Rt::spawn_local(async move {
            let commander_local_state = Arc::new(ui_updaters::CommanderLocalState::new(
                sender_to_commander,
                sender_to_process_manager,
            ));

            loop {
                let message = receiver.recv().await.unwrap();
                mbg!(&message);

                let cache = cache.clone();
                let commander_local_state = commander_local_state.clone();

                At::Rt::spawn_local(async move {
                    match message {
                        ui_model::Message::CloseError => {
                            model.external_errors.reset();
                        }
                        ui_model::Message::SignIn(msg) => {
                            msg.update(model, cache, commander_local_state).await
                        }
                        ui_model::Message::SignUp(msg) => {
                            msg.update(model, cache, commander_local_state).await
                        }
                        ui_model::Message::CompanyAndBranchSelection(msg) => {
                            msg.update(model, cache, commander_local_state).await
                        }
                        ui_model::Message::CreateCompany(msg) => {
                            msg.update(model, cache, commander_local_state).await
                        }
                        ui_model::Message::CreateCompanyBranch(msg) => {
                            msg.update(model, cache, commander_local_state).await
                        }
                    }
                });
            }
        });
    }
}

fn listen_to_error_actor<At: AllClientTypes>(
    mut receiver_to_error: <At::Mpsc as MultiProducerSingleConsumer>::Receiver<
        db_types::HashimError,
    >,
    external_errors_signal: &'static At::StringVec,
) {
    At::Rt::spawn_local(async move {
        loop {
            let err = receiver_to_error.recv().await.unwrap();
            external_errors_signal.set(err.to_string());
        }
    });
}
