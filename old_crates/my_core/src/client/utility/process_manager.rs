use crate::client::utility::ui_model::AllSignalTypes;
use crate::client::utility::ui_model::Dialog;
use crate::client::utility::ui_model::HashimSignal;
use crate::client::utility::ui_model::UserConsent;
use crate::utility::traits::JoinHandle;
use crate::utility::traits::MultiProducerSingleConsumer;
use crate::utility::traits::Receiver;
use crate::utility::traits::Runtime;
use crate::utility::traits::Sender;
use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum ProcessName {
    SignIn,
    SignUp,
    CreateCompanyBranch,
    CreateAccount,
    CreateAccountForBranch,
    CreateJournalEntry,
}

pub(crate) enum MessageFromProcess<Mpsc: MultiProducerSingleConsumer, As: AllSignalTypes> {
    Subscribe {
        sender: Mpsc::Sender<MessageToProcess>,
        dialog: &'static As::Dialog,
    },
    Response {
        is_response_from_server: bool,
        is_response_ok:          bool,
    },
}

pub(crate) enum MessageToProcessManager<Mpsc: MultiProducerSingleConsumer, As: AllSignalTypes> {
    FromUser {
        process_name: ProcessName,
        consent:      UserConsent,
    },
    FromProcess {
        process_name: ProcessName,
        message:      MessageFromProcess<Mpsc, As>,
    },
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum MessageToProcess {
    FallBackToCache,
    CancelOperation,
}

pub(crate) fn process_manager_actor<
    Mpsc: MultiProducerSingleConsumer,
    As: AllSignalTypes,
    Rt: Runtime,
>() -> Mpsc::Sender<MessageToProcessManager<Mpsc, As>> {
    let (sender, mut receiver) = Mpsc::channel();

    Rt::spawn_local(async move {
        struct ProcessInfo<Rt: Runtime, Mpsc: MultiProducerSingleConsumer, As: AllSignalTypes> {
            sender:                  Mpsc::Sender<MessageToProcess>,
            dialog:                  &'static As::Dialog,
            timer_handle:            Rt::JoinHandle<()>,
            is_response_from_server: Option<bool>,
            is_ok:                   Option<bool>,
            is_user_want_to_proceed: UserConsent,
        }

        let mut process_states = HashMap::<ProcessName, ProcessInfo<Rt, Mpsc, As>>::new();

        loop {
            let msg = receiver.recv().await.unwrap();

            match msg {
                MessageToProcessManager::FromUser {
                    process_name,
                    consent,
                } => {
                    let table = process_states.get_mut(&process_name).unwrap();

                    table.dialog.set(Dialog::Hide);
                    table.is_user_want_to_proceed = consent;
                    table.timer_handle.abort().await;

                    match consent {
                        UserConsent::WaitForServerResponse => {
                            table.timer_handle = timer_handle::<Rt, As>(table.dialog);
                        }
                        UserConsent::DontWaitForServerResponse => {
                            table.sender.send(MessageToProcess::FallBackToCache).await.unwrap();
                        }
                        UserConsent::CancelOperation => {
                            table.sender.send(MessageToProcess::CancelOperation).await.unwrap();
                        }
                    };
                }
                MessageToProcessManager::FromProcess {
                    process_name,
                    message,
                } => {
                    match message {
                        MessageFromProcess::Subscribe {
                            sender,
                            dialog,
                        } => {
                            let timer_handle = timer_handle::<Rt, As>(dialog);

                            process_states.insert(process_name, ProcessInfo {
                                sender,
                                dialog,
                                timer_handle,
                                is_response_from_server: None,
                                is_ok: None,
                                is_user_want_to_proceed: UserConsent::WaitForServerResponse,
                            });
                        }
                        MessageFromProcess::Response {
                            is_response_from_server,
                            is_response_ok,
                        } => {
                            let table = process_states.get_mut(&process_name).unwrap();

                            table.is_ok = Some(is_response_ok);
                            table.is_response_from_server = Some(is_response_from_server);

                            if is_response_from_server {
                                table.sender.send(MessageToProcess::CancelOperation).await.unwrap();

                                process_states.remove(&process_name);
                            }
                        }
                    };
                }
            };
        }
    });

    sender
}

fn timer_handle<Rt: Runtime, As: AllSignalTypes>(
    dialog_clone: &'static As::Dialog,
) -> Rt::JoinHandle<()> {
    Rt::abortable_spawn_local(async move {
        Rt::sleep(Duration::from_secs(5)).await;
        dialog_clone.set(Dialog::Show);
    })
}
