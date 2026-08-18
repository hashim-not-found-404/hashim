use crate::accounting_client::client_domain::process_manager;
use crate::accounting_client::client_domain::ui_model;
use crate::utility::traits;
use std::sync::Mutex;

pub(crate) struct CommanderLocalState<
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
> {
    pub(crate) sender_to_process_manager:
        Mutex<Mpsc::Sender<process_manager::MessageToProcessManager<Mpsc, As>>>,
    pub(crate) aborter_to_company_and_branch_listener:        Aborter,
    pub(crate) aborter_to_create_account_for_branch_listener: Aborter,
    pub(crate) aborter_to_create_journal_entry_listener:      Aborter,
}

impl<Mpsc: traits::MultiProducerSingleConsumer, As: ui_model::AllSignalTypes>
    CommanderLocalState<Mpsc, As>
{
    pub(crate) fn new(
        sender_to_process_manager: Mpsc::Sender<process_manager::MessageToProcessManager<Mpsc, As>>,
    ) -> Self {
        CommanderLocalState {
            sender_to_process_manager:                     Mutex::new(sender_to_process_manager),
            aborter_to_company_and_branch_listener:        Aborter::default(),
            aborter_to_create_account_for_branch_listener: Aborter::default(),
            aborter_to_create_journal_entry_listener:      Aborter::default(),
        }
    }
}

#[derive(Default)]
pub(crate) struct Aborter(Mutex<Option<Box<dyn FnOnce()>>>);

impl Aborter {
    pub(crate) fn set(&self, new_aborter: Box<dyn FnOnce()>) {
        let mut guard = self.0.lock().unwrap();
        match *guard {
            Some(_) => {
                unreachable!(
                    "this should not happen but it happen because you didn't abort the listener befor start new one or you have two listener in one field"
                )
            }
            None => *guard = Some(new_aborter),
        }
    }

    pub(crate) fn abort(&self) {
        let mut guard = self.0.lock().unwrap();
        if let Some(f) = guard.take() {
            f();
        }
    }
}
