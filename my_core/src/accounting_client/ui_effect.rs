use crate::{
    accounting_client::{
        cache_actor,
        client_traits::{self, AllSignalTypes, HashimSignal},
        process_manager, ui_model,
        ui_updaters::{self, Mvu},
    },
    accounting_domain::{cases, types},
    mbg,
    utility::traits::{self, MultiProducerSingleConsumer, Receiver, Runtime, Sender},
};
use std::sync::Arc;

pub struct Commander<Mpsc: traits::MultiProducerSingleConsumer> {
    sender: <Mpsc as MultiProducerSingleConsumer>::Sender<ui_model::Message>,
}

impl<Mpsc: traits::MultiProducerSingleConsumer> Clone for Commander<Mpsc> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<Mpsc: traits::MultiProducerSingleConsumer> Commander<Mpsc> {
    pub(crate) fn new<
        As: AllSignalTypes,
        Rt: traits::Runtime,
        Rn: traits::RandomNumber,
        Id: cases::RowId,
        Rg: traits::Regex,
    >(
        receiver_to_error: <Mpsc as MultiProducerSingleConsumer>::Receiver<types::HashimError>,
        sender_to_process_manager: <Mpsc as MultiProducerSingleConsumer>::Sender<
            process_manager::MessageToProcessManager<Mpsc, As>,
        >,
        model: &'static ui_model::Model<As>,
        cache: cache_actor::CacheStruct<Mpsc>,
    ) -> Self {
        let (sender_to_commander, receiver_to_commander) = Mpsc::channel();

        listen_to_error_actor::<Rt, Mpsc, As>(receiver_to_error, &model.external_errors);

        Self::commander_actor::<As, Rt, Rn, Id, Rg>(
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

    pub fn send<Rt: traits::Runtime>(&self, msg: ui_model::Message) {
        let mut sender = self.sender.clone();
        Rt::spawn_local(async move {
            sender.send(msg).await.unwrap();
        });
    }

    fn commander_actor<
        As: AllSignalTypes,
        Rt: traits::Runtime,
        Rn: traits::RandomNumber,
        Id: cases::RowId,
        Rg: traits::Regex,
    >(
        mut receiver: <Mpsc as MultiProducerSingleConsumer>::Receiver<ui_model::Message>,
        sender_to_commander: <Mpsc as MultiProducerSingleConsumer>::Sender<ui_model::Message>,
        sender_to_process_manager: <Mpsc as MultiProducerSingleConsumer>::Sender<
            process_manager::MessageToProcessManager<Mpsc, As>,
        >,
        model: &'static ui_model::Model<As>,
        cache: cache_actor::CacheStruct<Mpsc>,
    ) {
        Rt::spawn_local(async move {
            let commander_local_state = Arc::new(ui_updaters::CommanderLocalState::new(
                sender_to_commander,
                sender_to_process_manager,
            ));

            loop {
                let message = receiver.recv().await.unwrap();
                mbg!(&message);

                let cache = cache.clone();
                let commander_local_state = commander_local_state.clone();

                Rt::spawn_local(async move {
                    match message {
                        ui_model::Message::CloseError => {
                            model.external_errors.reset();
                        }
                        ui_model::Message::SignIn(msg) => {
                            msg.update::<Rn, Rt, Id, Mpsc, Rg, As>(
                                model,
                                cache,
                                commander_local_state,
                            )
                            .await
                        }
                        ui_model::Message::SignUp(msg) => {
                            msg.update::<Rn, Rt, Id, Mpsc, Rg, As>(
                                model,
                                cache,
                                commander_local_state,
                            )
                            .await
                        }
                        ui_model::Message::CompanyAndBranchSelection(msg) => {
                            msg.update::<Rn, Rt, Id, Mpsc, Rg, As>(
                                model,
                                cache,
                                commander_local_state,
                            )
                            .await
                        }
                        ui_model::Message::CreateCompany(msg) => {
                            msg.update::<Rn, Rt, Id, Mpsc, Rg, As>(
                                model,
                                cache,
                                commander_local_state,
                            )
                            .await
                        }
                        ui_model::Message::CreateCompanyBranch(msg) => {
                            msg.update::<Rn, Rt, Id, Mpsc, Rg, As>(
                                model,
                                cache,
                                commander_local_state,
                            )
                            .await
                        }
                    }
                });
            }
        });
    }
}

fn listen_to_error_actor<
    Rt: traits::Runtime,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: client_traits::AllSignalTypes,
>(
    mut receiver_to_error: <Mpsc as MultiProducerSingleConsumer>::Receiver<types::HashimError>,
    external_errors_signal: &'static As::StringVec,
) {
    Rt::spawn_local(async move {
        loop {
            let err = receiver_to_error.recv().await.unwrap();
            external_errors_signal.set(err.to_string());
        }
    });
}
