use anyhow::Context;
use anyhow::Result;
use infrastructure::actors::MpscReceiver;
use infrastructure::actors::MpscSender;
use infrastructure::actors::Receiver;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use infrastructure::time::Ti;
use infrastructure::time::Time;
use kernel::types::HashimError;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::Arc;
use utility::cache::CacheStruct;
use utility::process_manager::MessageToProcessManager;
use utility::ui_effect::Aborters;
use utility::ui_effect::MessageTrait;
use utility_ui::domain::HashimSignal;

pub fn spawn_listener(
    mut receiver_to_error: MpscReceiver<anyhow::Error>,
    external_errors_signal: Arc<impl LocalModel>,
) {
    Rt::spawn_local(async move {
        let error = external_errors_signal.errors();

        loop {
            let new_err = receiver_to_error.recv().await.unwrap();
            let back_trace = match new_err.downcast_ref::<HashimError>() {
                Some(_) => None,
                None => {
                    let bt = new_err.backtrace().to_string();

                    if bt.is_empty() || bt.contains("disabled") || bt.contains("unsupported") {
                        None
                    } else {
                        Some(bt)
                    }
                }
            };

            let error_name = new_err.to_string();
            let time_unix_ms = Ti::now_as_unix_milliseconds();

            let mut errors = error.read();

            match errors.0.iter_mut().find(|e| e.name == error_name) {
                Some(entry) => {
                    if entry.back_trace.is_none() {
                        entry.back_trace = back_trace;
                    }
                    entry.number_of_errors = entry.number_of_errors.saturating_add(1);
                    entry.time_unix_ms = time_unix_ms;
                }
                None => {
                    errors.0.push(ErrorInfo {
                        name: error_name,
                        number_of_errors: 1,
                        time_unix_ms,
                        back_trace,
                        is_expand: false,
                    });
                }
            }

            errors.0.sort_by_key(|e| e.time_unix_ms);

            error.set(errors);
        }
    });
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct ErrorInfo {
    pub name: String,
    pub number_of_errors: u8,
    pub time_unix_ms: u64,
    pub back_trace: Option<String>,

    pub is_expand: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct ErrorList(pub Vec<ErrorInfo>);

#[derive(Debug, Clone)]
pub enum Message {
    DeleteAll,
    DeleteOne(usize),
    ExpandOrCollapseAll,
    ExpandOrCollapseOne(usize),
}

impl MessageTrait for Message {}

pub trait GlobalModel: 'static {}

pub trait LocalModel: 'static {
    fn is_expand_all(&self) -> impl HashimSignal<bool>;
    fn errors(&self) -> impl HashimSignal<ErrorList>;
}

pub async fn update_generic(
    message: Message,
    global_model: Arc<impl GlobalModel>,
    local_model: Arc<impl LocalModel>,
    cache: CacheStruct,
    mut sender_to_process_manager: MpscSender<MessageToProcessManager>,
    aborters: Aborters,
) -> Result<()> {
    match message {
        Message::DeleteOne(i) => {
            let mut err = local_model.errors().read();

            err.0.remove(i);

            local_model.errors().set(err);
        }
        Message::DeleteAll => {
            local_model.errors().reset();
        }
        Message::ExpandOrCollapseAll => {
            let a = local_model.is_expand_all().read();

            local_model.is_expand_all().set(a ^ true);
        }
        Message::ExpandOrCollapseOne(i) => {
            let mut err = local_model.errors().read();

            let a = err.0.get_mut(i).context("context")?;
            a.is_expand ^= true;

            local_model.errors().set(err);
        }
    }

    Ok(())
}
