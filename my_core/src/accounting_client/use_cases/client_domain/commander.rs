use crate::{
    accounting_client::use_cases::client_domain::{process_manager, ui_model},
    accounting_domain::{self, cases::utility::types},
    utility::traits,
};
use std::sync::Mutex;

pub(crate) struct CommanderLocalState<
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
> {
    pub(crate) sender_to_commander: Mutex<Mpsc::Sender<ui_model::Message>>,
    pub(crate) sender_to_process_manager:
        Mutex<Mpsc::Sender<process_manager::MessageToProcessManager<Mpsc, As>>>,
    pub(crate) user_uuid: Mutex<Option<types::UuidType>>,
    pub(crate) selected_company_branch: Mutex<Option<types::UuidType>>,
    pub(crate) aborter_to_company_and_branch_listener: Mutex<Option<Box<dyn FnOnce()>>>,
}

impl<Mpsc: traits::MultiProducerSingleConsumer, As: ui_model::AllSignalTypes>
    CommanderLocalState<Mpsc, As>
{
    pub(crate) fn new(
        sender_to_commander: Mpsc::Sender<ui_model::Message>,
        sender_to_process_manager: Mpsc::Sender<process_manager::MessageToProcessManager<Mpsc, As>>,
    ) -> Self {
        CommanderLocalState {
            sender_to_commander: Mutex::new(sender_to_commander),
            sender_to_process_manager: Mutex::new(sender_to_process_manager),
            user_uuid: Mutex::default(),
            selected_company_branch: Mutex::default(),
            aborter_to_company_and_branch_listener: Mutex::default(),
        }
    }
}
