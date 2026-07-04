use crate::{
    traits::{AllClientTypes, JoinHandle, MultiProducerSingleConsumer, Receiver, Runtime, Sender},
    ui_model::{self, HashimSignal},
};
use std::{collections::HashMap, hash::Hash, time::Duration};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum ProcessName {
    SignIn,
    SignUp,
    CreateCompanyBranch,
}

pub(crate) enum Event<At: AllClientTypes> {
    Subscribe {
        sender: <At::Mpsc as MultiProducerSingleConsumer>::Sender<ProceedResult>,
        dialog: &'static At::Dialog,
    },
    GotResponseFromCache {
        is_response_ok: bool,
    },
    Completed {
        is_response_ok: bool,
    },
}

pub(crate) enum MessageToProcessManager<At: AllClientTypes> {
    FromUser {
        process_name: ProcessName,
        consent: UserConsent,
    },
    FromProcess {
        process_name: ProcessName,
        event: Event<At>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ProceedResult {
    Yes,
    No,
}

#[derive(Debug, Clone, Copy)]
pub enum UserConsent {
    WaitForServerResponse,
    DontWaitForServerResponse,
}

#[derive(Debug, Clone, Copy)]
pub enum IsProceed {
    Wait,
    Yes,
    No,
}

pub(crate) fn is_proceed(
    is_ok: bool,
    is_response_from_server: bool,
    is_user_want_to_proceed: UserConsent,
) -> IsProceed {
    match (is_response_from_server, is_ok, is_user_want_to_proceed) {
        (true, true, UserConsent::WaitForServerResponse) => IsProceed::Yes,
        (true, true, UserConsent::DontWaitForServerResponse) => IsProceed::Yes,
        (true, false, UserConsent::WaitForServerResponse) => IsProceed::No,
        (true, false, UserConsent::DontWaitForServerResponse) => IsProceed::No,
        (false, true, UserConsent::WaitForServerResponse) => IsProceed::Wait,
        (false, true, UserConsent::DontWaitForServerResponse) => IsProceed::Yes,
        (false, false, UserConsent::WaitForServerResponse) => IsProceed::Wait,
        (false, false, UserConsent::DontWaitForServerResponse) => IsProceed::Yes,
    }
}

pub(crate) fn process_manager_actor<At: AllClientTypes>()
-> <At::Mpsc as MultiProducerSingleConsumer>::Sender<MessageToProcessManager<At>> {
    let (sender, mut receiver) = At::Mpsc::channel();

    At::Rt::spawn_local(async move {
        struct ProcessInfo<At: AllClientTypes> {
            sender: <At::Mpsc as MultiProducerSingleConsumer>::Sender<ProceedResult>,
            dialog: &'static At::Dialog,
            timer_handle: <At::Rt as Runtime>::JoinHandle<()>,
            is_response_from_server: Option<bool>,
            is_ok: Option<bool>,
            is_user_want_to_proceed: UserConsent,
        }

        let mut process_states = HashMap::<ProcessName, ProcessInfo<At>>::new();

        loop {
            let msg = receiver.recv().await.unwrap();

            let process_name = match msg {
                MessageToProcessManager::FromUser {
                    process_name,
                    consent,
                } => {
                    let table = process_states.get_mut(&process_name).unwrap();

                    table.dialog.set(ui_model::Dialog::Hide);
                    table.is_user_want_to_proceed = consent;
                    table.timer_handle.abort().await;

                    match consent {
                        UserConsent::WaitForServerResponse => {
                            table.timer_handle = timer_handle::<At>(&table.dialog);
                        }
                        UserConsent::DontWaitForServerResponse => {}
                    };

                    process_name
                }
                MessageToProcessManager::FromProcess {
                    process_name,
                    event,
                } => {
                    match event {
                        Event::Subscribe { sender, dialog } => {
                            let timer_handle = timer_handle::<At>(&dialog);

                            process_states.insert(
                                process_name,
                                ProcessInfo {
                                    sender,
                                    dialog,
                                    timer_handle,
                                    is_response_from_server: None,
                                    is_ok: None,
                                    is_user_want_to_proceed: UserConsent::WaitForServerResponse,
                                },
                            );
                        }
                        Event::GotResponseFromCache { is_response_ok } => {
                            process_states.entry(process_name).and_modify(|table| {
                                table.is_response_from_server = Some(false);
                                table.is_ok = Some(is_response_ok);
                            });
                        }
                        Event::Completed { is_response_ok } => {
                            process_states.entry(process_name).and_modify(|table| {
                                table.is_response_from_server = Some(true);
                                table.is_ok = Some(is_response_ok);
                            });
                        }
                    };
                    process_name
                }
            };

            let process_info = process_states.get_mut(&process_name).unwrap();

            if let (Some(is_ok), Some(is_response_from_server)) =
                (process_info.is_ok, process_info.is_response_from_server)
            {
                let proceed = is_proceed(
                    is_ok,
                    is_response_from_server,
                    process_info.is_user_want_to_proceed,
                );

                match proceed {
                    IsProceed::Wait => {}
                    IsProceed::Yes => {
                        process_info.timer_handle.abort().await;
                        process_info.dialog.set(ui_model::Dialog::Hide);
                        process_info.sender.send(ProceedResult::Yes).await.unwrap();
                        process_states.remove(&process_name);
                    }
                    IsProceed::No => {
                        process_info.timer_handle.abort().await;
                        process_info.dialog.set(ui_model::Dialog::Hide);
                        process_info.sender.send(ProceedResult::No).await.unwrap();
                        process_states.remove(&process_name);
                    }
                }
            }
        }
    });

    sender
}

fn timer_handle<At: AllClientTypes>(
    dialog_clone: &'static <At as AllClientTypes>::Dialog,
) -> <At::Rt as Runtime>::JoinHandle<()> {
    At::Rt::abortable_spawn_local(async move {
        At::Rt::sleep(Duration::from_secs(5)).await;
        dialog_clone.set(ui_model::Dialog::Show);
    })
}
