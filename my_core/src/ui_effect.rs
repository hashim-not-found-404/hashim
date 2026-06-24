use crate::{prelude::*, ui_updaters::Mvu};

pub(crate) struct CommanderLocalState<As: AllSignalTypes, Mpsc: MultiProducerSingleConsumer> {
    pub(crate) sender_to_commander: Mutex<Mpsc::Sender<ui_model::Message>>,
    pub(crate) sender_to_process_manager:
        Mutex<Mpsc::Sender<process_manager::MessageToProcessManager<As, Mpsc>>>,
    pub(crate) user_uuid: Mutex<Option<db_types::UuidType>>,
    pub(crate) selected_company_branch: Mutex<Option<db_types::UuidType>>,
    pub(crate) aborter_to_company_and_branch_listener: Mutex<Option<Box<dyn FnOnce()>>>,
}

pub struct Commander<As: AllSignalTypes, At: AllClientTypes, Mpsc: MultiProducerSingleConsumer> {
    _ph: PhantomData<(At, As)>,
    sender: Mpsc::Sender<ui_model::Message>,
}

impl<As: AllSignalTypes, At: AllClientTypes, Mpsc: MultiProducerSingleConsumer> Clone
    for Commander<As, At, Mpsc>
{
    fn clone(&self) -> Self {
        Self {
            _ph: self._ph.clone(),
            sender: self.sender.clone(),
        }
    }
}

impl<
    As: AllSignalTypes + 'static,
    At: AllClientTypes + 'static,
    Mpsc: MultiProducerSingleConsumer + 'static,
> Commander<As, At, Mpsc>
{
    pub(crate) fn new(
        receiver_to_error: Mpsc::Receiver<HashimError>,
        sender_to_process_manager: Mpsc::Sender<process_manager::MessageToProcessManager<As, Mpsc>>,
        model: ui_model::Model<As>,
        cache: cache_actor::Cache<At, Mpsc>,
    ) -> Self {
        let (sender_to_commands, receiver_to_commands) = Mpsc::channel();

        listen_to_error_actor::<As, At, Mpsc>(receiver_to_error, model.external_errors.clone());

        Self::commands_actor(
            receiver_to_commands,
            sender_to_commands.clone(),
            sender_to_process_manager,
            model,
            cache,
        );

        Self {
            _ph: PhantomData,
            sender: sender_to_commands,
        }
    }

    pub fn send(mut self, msg: ui_model::Message) {
        At::Rt::spawn_local(async move {
            self.sender.send(msg).await.unwrap();
        });
    }

    fn commands_actor(
        mut receiver: Mpsc::Receiver<ui_model::Message>,
        sender_to_commands: Mpsc::Sender<ui_model::Message>,
        sender_to_process_manager: Mpsc::Sender<process_manager::MessageToProcessManager<As, Mpsc>>,
        model: ui_model::Model<As>,
        cache: cache_actor::Cache<At, Mpsc>,
    ) {
        let commander_local_state = Arc::new(CommanderLocalState {
            sender_to_commander: Mutex::new(sender_to_commands),
            sender_to_process_manager: Mutex::new(sender_to_process_manager),
            user_uuid: Mutex::default(),
            selected_company_branch: Mutex::default(),
            aborter_to_company_and_branch_listener: Mutex::default(),
        });

        At::Rt::spawn_local(async move {
            loop {
                let message = receiver.recv().await.unwrap();

                let model = model.clone();
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

fn listen_to_error_actor<
    As: AllSignalTypes + 'static,
    At: AllClientTypes + 'static,
    Mpsc: MultiProducerSingleConsumer + 'static,
>(
    mut receiver_to_error: Mpsc::Receiver<HashimError>,
    external_errors_signal: As::StringVec,
) {
    At::Rt::spawn_local(async move {
        loop {
            let err = receiver_to_error.recv().await.unwrap();
            external_errors_signal.set(err.to_string());
        }
    });
}
