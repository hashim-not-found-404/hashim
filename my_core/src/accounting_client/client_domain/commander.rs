use crate::accounting_client::client_domain::process_manager;
use crate::accounting_client::client_domain::ui_model;
use crate::mbg;
use crate::utility::traits;
use std::sync::Mutex;

pub(crate) struct CommanderLocalState<
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
> {
    pub(crate) sender_to_process_manager:
        Mutex<Mpsc::Sender<process_manager::MessageToProcessManager<Mpsc, As>>>,
    pub(crate) aborter_to_company_and_branch_listener: Aborter,
    pub(crate) aborter_to_accounts_listener:           Aborter,
}

impl<Mpsc: traits::MultiProducerSingleConsumer, As: ui_model::AllSignalTypes>
    CommanderLocalState<Mpsc, As>
{
    pub(crate) fn new(
        sender_to_process_manager: Mpsc::Sender<process_manager::MessageToProcessManager<Mpsc, As>>,
    ) -> Self {
        CommanderLocalState {
            sender_to_process_manager:              Mutex::new(sender_to_process_manager),
            aborter_to_company_and_branch_listener: Aborter::default(),
            aborter_to_accounts_listener:           Aborter::default(),
        }
    }
}

#[derive(Default)]
pub(crate) struct Aborter(Mutex<Option<Box<dyn FnOnce()>>>);

impl Aborter {
    pub(crate) fn set(&self, new_aborter: Box<dyn FnOnce()>) {
        self.abort();
        *self.0.lock().unwrap() = Some(new_aborter);
    }

    pub(crate) fn abort(&self) {
        let mut guard = self.0.lock().unwrap();
        if let Some(f) = guard.take() {
            f();
        }
    }
}
