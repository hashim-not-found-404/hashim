use crate::accounting_client::cache_op;
use crate::accounting_client::client_domain::cache;
use crate::accounting_client::client_domain::client_traits;
use crate::accounting_client::client_domain::commander;
use crate::accounting_client::client_domain::process_manager;
use crate::accounting_client::client_domain::ui_model;
use crate::accounting_client::client_domain::ui_model::AllSignalTypes;
use crate::accounting_client::client_domain::ui_model::HashimSignal;
use crate::accounting_domain::utility::types;
use crate::mbg;
use crate::utility::traits;
use crate::utility::traits::Receiver;
use crate::utility::traits::Sender;
use std::sync::Arc;

pub struct Commander<Mpsc: traits::MultiProducerSingleConsumer> {
    sender: Mpsc::Sender<ui_model::Message>,
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
        Id: types::RowId,
        Ti: traits::Time,
        Ch: cache::Cache + 'static,
        Dbb: cache_op::DbBundle<Ch>,
    >(
        receiver_to_error: Mpsc::Receiver<types::HashimError>,
        sender_to_process_manager: Mpsc::Sender<process_manager::MessageToProcessManager<Mpsc, As>>,
        model: &'static ui_model::Model<As>,
        cache: client_traits::CacheActorStruct<Mpsc>,
    ) -> Self {
        let (sender_to_commander, receiver_to_commander) = Mpsc::channel();

        listen_to_error_actor::<Rt, Mpsc, As>(receiver_to_error, &model.external_errors);

        Self::commander_actor::<As, Rt, Rn, Id, Ti, Ch, Dbb>(
            receiver_to_commander,
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
        Id: types::RowId,
        Ti: traits::Time,
        Ch: cache::Cache + 'static,
        Dbb: cache_op::DbBundle<Ch>,
    >(
        mut receiver: Mpsc::Receiver<ui_model::Message>,
        sender_to_process_manager: Mpsc::Sender<process_manager::MessageToProcessManager<Mpsc, As>>,
        model: &'static ui_model::Model<As>,
        cache: client_traits::CacheActorStruct<Mpsc>,
    ) {
        Rt::spawn_local(async move {
            let commander_local_state =
                Arc::new(commander::CommanderLocalState::new(sender_to_process_manager));

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
                            msg.update::<Rn, Rt, Mpsc, As, Ch, Dbb::SignIn>(
                                model,
                                cache,
                                commander_local_state,
                            )
                            .await
                        }
                        ui_model::Message::SignUp(msg) => {
                            msg.update::<Rn, Rt, Id, Mpsc, As, Ch, Dbb::SignUp>(
                                model,
                                cache,
                                commander_local_state,
                            )
                            .await
                        }
                        ui_model::Message::CompanyAndBranchSelection(msg) => {
                            msg.update::<Rn, Rt, Mpsc, As, Ch, Dbb::ListCompanyAndBranch>(
                                model,
                                cache,
                                commander_local_state,
                            )
                            .await
                        }
                        ui_model::Message::CreateCompany(msg) => {
                            msg.update::<Rn, Id, Mpsc, As, Ch, Dbb::CreateCompany>(
                                model,
                                cache,
                                commander_local_state,
                            )
                            .await
                        }
                        ui_model::Message::CreateCompanyBranch(msg) => {
                            msg.update::<Rn, Rt, Id, Mpsc,  As, Ch, Dbb::CreateCompanyBranch>(
                                model,
                                cache,
                                commander_local_state,
                            )
                            .await
                        }
                        ui_model::Message::Home(msg) => {
                            msg.update::<Mpsc,  As>(
                                model,
                                cache,
                                commander_local_state,
                            )
                            .await
                        }
                        ui_model::Message::CreateAccount(msg) => {
                            msg.update::<Rn, Rt, Id, Mpsc, As, Ch, Dbb::CreateAccount>(
                                model,
                                cache,
                                commander_local_state,
                            )
                            .await
                        }
                        ui_model::Message::CreateAccountForBranch(msg) => {
                            msg.update::<Rn, Rt, Id, Mpsc, As, Ch, Dbb::CreateAccountForBranch,Dbb::GetAllAccountsForBranch>(
                                model,
                                cache,
                                commander_local_state,
                            )
                            .await
                        }
                        ui_model::Message::CreateJournalEntry(msg) => {
                            msg.update::<Rn, Rt, Id, Mpsc,  Ti, As, Ch, Dbb::CreateJournalEntry,Dbb::GetAllAccountsForBranch>(
                                model,
                                cache,
                                commander_local_state,
                            ).await
                        }}
                });
            }
        });
    }
}

fn listen_to_error_actor<
    Rt: traits::Runtime,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
>(
    mut receiver_to_error: Mpsc::Receiver<types::HashimError>,
    external_errors_signal: &'static As::StringVec,
) {
    Rt::spawn_local(async move {
        loop {
            let err = receiver_to_error.recv().await.unwrap();
            external_errors_signal.set(err.to_string());
        }
    });
}
