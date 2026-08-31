use adapters::actors::MultiProducerSingleConsumer;
use adapters::actors::Receiver;
use adapters::actors::Sender;
use adapters::runtime::JoinHandle;
use adapters::runtime::Runtime;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;

pub trait Dialog: 'static {
    fn show(&self);
    fn hide(&self);
}

#[derive(Debug, Clone, Copy)]
pub enum UserConsent {
    WaitForServerResponse,
    DontWaitForServerResponse,
    CancelOperation,
}

pub enum MessageFromProcess<Mpsc: MultiProducerSingleConsumer, Di: Dialog> {
    Subscribe {
        sender: Mpsc::Sender<MessageToProcess>,
        dialog: &'static Di,
    },
    Response {
        is_response_from_server: bool,
        is_response_ok:          bool,
    },
}

pub enum MessageToProcessManager<
    Mpsc: MultiProducerSingleConsumer,
    Di: Dialog,
    ProcessName: Clone + Copy + Eq + Hash + PartialEq + 'static,
> {
    FromUser {
        process_name: ProcessName,
        consent:      UserConsent,
    },
    FromProcess {
        process_name: ProcessName,
        message:      MessageFromProcess<Mpsc, Di>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum MessageToProcess {
    FallBackToCache,
    CancelOperation,
}

pub fn process_manager_actor<
    Mpsc: MultiProducerSingleConsumer,
    Di: Dialog,
    Rt: Runtime,
    ProcessName: Clone + Copy + Eq + Hash + PartialEq + 'static,
>() -> Mpsc::Sender<MessageToProcessManager<Mpsc, Di, ProcessName>> {
    let (sender, mut receiver) = Mpsc::channel();

    Rt::spawn_local(async move {
        struct ProcessInfo<Rt: Runtime, Mpsc: MultiProducerSingleConsumer, Di: Dialog> {
            sender:                  Mpsc::Sender<MessageToProcess>,
            dialog:                  &'static Di,
            timer_handle:            Rt::JoinHandle<()>,
            is_response_from_server: Option<bool>,
            is_ok:                   Option<bool>,
            is_user_want_to_proceed: UserConsent,
        }

        let mut process_states = HashMap::<ProcessName, ProcessInfo<Rt, Mpsc, Di>>::new();

        loop {
            let msg = receiver.recv().await.unwrap();

            match msg {
                MessageToProcessManager::FromUser {
                    process_name,
                    consent,
                } => {
                    let table = process_states.get_mut(&process_name).unwrap();

                    table.dialog.hide();
                    table.is_user_want_to_proceed = consent;
                    table.timer_handle.abort().await;

                    match consent {
                        UserConsent::WaitForServerResponse => {
                            table.timer_handle = timer_handle::<Rt, Di>(table.dialog);
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
                            let timer_handle = timer_handle::<Rt, Di>(dialog);

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

fn timer_handle<Rt: Runtime, Di: Dialog>(dialog_clone: &'static Di) -> Rt::JoinHandle<()> {
    Rt::abortable_spawn_local(async move {
        Rt::sleep(Duration::from_secs(5)).await;
        dialog_clone.show();
    })
}
