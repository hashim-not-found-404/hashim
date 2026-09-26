use anyhow::Context;
use anyhow::Result;
use infrastructure::actors::Mpsc;
use infrastructure::actors::MpscReceiver;
use infrastructure::actors::MpscSender;
use infrastructure::actors::MultiProducerSingleConsumer;
use infrastructure::actors::Receiver;
use infrastructure::actors::Sender;
use infrastructure::random_number::RandomNumber;
use infrastructure::random_number::Rn;
use infrastructure::runtime::Jh;
use infrastructure::runtime::JoinHandle;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ProcessId(u16);

impl Default for ProcessId {
    fn default() -> Self {
        ProcessId(Rn::generate() as u16)
    }
}

pub trait Dialog {
    fn show(&self);
    fn hide(&self);
}

pub type DialogType = Arc<dyn Dialog>;

#[derive(Debug, Clone, Copy)]
pub enum UserConsent {
    WaitForServerResponse,
    DontWaitForServerResponse,
    CancelOperation,
}

pub enum MessageFromProcess {
    Subscribe {
        sender: MpscSender<MessageToProcess>,
        dialog: DialogType,
    },
    Response {
        is_response_from_server: bool,
        is_response_ok: bool,
    },
}

pub enum MessageToProcessManager {
    FromUser {
        process_id: ProcessId,
        consent: UserConsent,
    },
    FromProcess {
        process_id: ProcessId,
        message: MessageFromProcess,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum MessageToProcess {
    FallBackToCache,
    CancelOperation,
}

pub fn process_manager_actor() -> MpscSender<MessageToProcessManager> {
    let (sender, mut receiver): (
        MpscSender<MessageToProcessManager>,
        MpscReceiver<MessageToProcessManager>,
    ) = Mpsc::channel();

    Rt::spawn_local(async move {
        struct ProcessInfo {
            sender: MpscSender<MessageToProcess>,
            dialog: DialogType,
            timer_handle: Jh<()>,
            is_response_from_server: Option<bool>,
            is_ok: Option<bool>,
            is_user_want_to_proceed: UserConsent,
        }

        let mut process_states = HashMap::<ProcessId, ProcessInfo>::new();

        loop {
            let _: Result<()> = async {
                let msg: MessageToProcessManager = receiver.recv().await?;

                match msg {
                    MessageToProcessManager::FromUser {
                        process_id,
                        consent,
                    } => {
                        let table = process_states
                            .get_mut(&process_id)
                            .context("process id not found")?;

                        table.dialog.hide();
                        table.is_user_want_to_proceed = consent;
                        table.timer_handle.abort().await;

                        match consent {
                            UserConsent::WaitForServerResponse => {
                                table.timer_handle = timer_handle(table.dialog.clone());
                            }
                            UserConsent::DontWaitForServerResponse => {
                                table.sender.send(MessageToProcess::FallBackToCache).await?;
                            }
                            UserConsent::CancelOperation => {
                                table.sender.send(MessageToProcess::CancelOperation).await?;
                            }
                        }
                    }
                    MessageToProcessManager::FromProcess {
                        process_id,
                        message,
                    } => match message {
                        MessageFromProcess::Subscribe { sender, dialog } => {
                            let timer_handle = timer_handle(dialog.clone());

                            process_states.insert(
                                process_id,
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
                        MessageFromProcess::Response {
                            is_response_from_server,
                            is_response_ok,
                        } => {
                            let table = process_states
                                .get_mut(&process_id)
                                .context("process id not found")?;

                            table.is_ok = Some(is_response_ok);
                            table.is_response_from_server = Some(is_response_from_server);

                            if is_response_from_server {
                                table.sender.send(MessageToProcess::CancelOperation).await?;

                                process_states.remove(&process_id);
                            }
                        }
                    },
                }

                Ok(())
            }
            .await;
        }
    });

    sender
}

fn timer_handle(dialog: DialogType) -> Jh<()> {
    Rt::abortable_spawn_local(async move {
        Rt::sleep(Duration::from_secs(5)).await;
        dialog.show();
    })
}
