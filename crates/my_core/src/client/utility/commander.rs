use crate::client::utility::process_manager::MessageToProcessManager;
use crate::client::utility::ui_model::AllSignalTypes;
use crate::utility::traits::MultiProducerSingleConsumer;
use std::sync::Arc;
use std::sync::Mutex;

pub(crate) type CommanderLocalState<Mpsc, As> = Arc<State<Mpsc, As>>;

pub(crate) struct State<Mpsc: MultiProducerSingleConsumer, As: AllSignalTypes> {
    pub(crate) sender_to_process_manager: Mutex<Mpsc::Sender<MessageToProcessManager<Mpsc, As>>>,
    pub(crate) aborter_to_company_and_branch_listener:        Aborter,
    pub(crate) aborter_to_create_account_for_branch_listener: Aborter,
    pub(crate) aborter_to_create_journal_entry_listener:      Aborter,
}

pub(crate) fn new<Mpsc: MultiProducerSingleConsumer, As: AllSignalTypes>(
    sender_to_process_manager: Mpsc::Sender<MessageToProcessManager<Mpsc, As>>,
) -> CommanderLocalState<Mpsc, As> {
    Arc::new(State {
        sender_to_process_manager:                     Mutex::new(sender_to_process_manager),
        aborter_to_company_and_branch_listener:        Aborter::default(),
        aborter_to_create_account_for_branch_listener: Aborter::default(),
        aborter_to_create_journal_entry_listener:      Aborter::default(),
    })
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
