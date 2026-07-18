use crate::{
    accounting_client::use_cases::client_domain::ui_model::{self, HashimSignal},
    utility::traits::{self, JoinHandle, Receiver, Sender},
};
use std::{collections::HashMap, hash::Hash, time::Duration};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum ProcessName {
    SignIn,
    SignUp,
    CreateCompanyBranch,
    CreateAccount,
}

pub(crate) enum Event<Mpsc: traits::MultiProducerSingleConsumer, As: ui_model::AllSignalTypes> {
    Subscribe {
        sender: Mpsc::Sender<ProceedResult>,
        dialog: &'static As::Dialog,
    },
    GotResponseFromCache {
        is_response_ok: bool,
    },
    Completed {
        is_response_ok: bool,
    },
}

pub(crate) enum MessageToProcessManager<
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
> {
    FromUser {
        process_name: ProcessName,
        consent: ui_model::UserConsent,
    },
    FromProcess {
        process_name: ProcessName,
        event: Event<Mpsc, As>,
    },
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ProceedResult {
    Yes,
    No,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum IsProceed {
    Wait,
    Yes,
    No,
}

pub(crate) fn is_proceed(
    is_ok: bool,
    is_response_from_server: bool,
    is_user_want_to_proceed: ui_model::UserConsent,
) -> IsProceed {
    match (is_response_from_server, is_ok, is_user_want_to_proceed) {
        (true, true, ui_model::UserConsent::WaitForServerResponse) => IsProceed::Yes,
        (true, true, ui_model::UserConsent::DontWaitForServerResponse) => IsProceed::Yes,
        (true, false, ui_model::UserConsent::WaitForServerResponse) => IsProceed::No,
        (true, false, ui_model::UserConsent::DontWaitForServerResponse) => IsProceed::No,
        (false, true, ui_model::UserConsent::WaitForServerResponse) => IsProceed::Wait,
        (false, true, ui_model::UserConsent::DontWaitForServerResponse) => IsProceed::Yes,
        (false, false, ui_model::UserConsent::WaitForServerResponse) => IsProceed::Wait,
        (false, false, ui_model::UserConsent::DontWaitForServerResponse) => IsProceed::Yes,
    }
}

pub(crate) fn process_manager_actor<
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
    Rt: traits::Runtime,
>() -> Mpsc::Sender<MessageToProcessManager<Mpsc, As>> {
    let (sender, mut receiver) = Mpsc::channel();

    Rt::spawn_local(async move {
        struct ProcessInfo<
            Rt: traits::Runtime,
            Mpsc: traits::MultiProducerSingleConsumer,
            As: ui_model::AllSignalTypes,
        > {
            sender: Mpsc::Sender<ProceedResult>,
            dialog: &'static As::Dialog,
            timer_handle: Rt::JoinHandle<()>,
            is_response_from_server: Option<bool>,
            is_ok: Option<bool>,
            is_user_want_to_proceed: ui_model::UserConsent,
        }

        let mut process_states = HashMap::<ProcessName, ProcessInfo<Rt, Mpsc, As>>::new();

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
                        ui_model::UserConsent::WaitForServerResponse => {
                            table.timer_handle = timer_handle::<Rt, As>(&table.dialog);
                        }
                        ui_model::UserConsent::DontWaitForServerResponse => {}
                    };

                    process_name
                }
                MessageToProcessManager::FromProcess {
                    process_name,
                    event,
                } => {
                    match event {
                        Event::Subscribe { sender, dialog } => {
                            let timer_handle = timer_handle::<Rt, As>(&dialog);

                            process_states.insert(
                                process_name,
                                ProcessInfo {
                                    sender,
                                    dialog,
                                    timer_handle,
                                    is_response_from_server: None,
                                    is_ok: None,
                                    is_user_want_to_proceed:
                                        ui_model::UserConsent::WaitForServerResponse,
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

fn timer_handle<Rt: traits::Runtime, As: ui_model::AllSignalTypes>(
    dialog_clone: &'static As::Dialog,
) -> Rt::JoinHandle<()> {
    Rt::abortable_spawn_local(async move {
        Rt::sleep(Duration::from_secs(5)).await;
        dialog_clone.set(ui_model::Dialog::Show);
    })
}
