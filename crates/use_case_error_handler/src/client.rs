use anyhow::Result;
use infrastructure::actors::MpscReceiver;
use infrastructure::actors::Receiver;
use infrastructure::runtime::Rt;
use infrastructure::runtime::Runtime;
use infrastructure::time::Ti;
use infrastructure::time::Time;
use kernel::types::HashimError;
use serde::Deserialize;
use serde::Serialize;
use std::backtrace::BacktraceStatus;
use std::fmt::Debug;
use utility::cache::ProcessId;
use utility::ui_effect::Commander;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UiContext;

pub fn spawn_listener(mut receiver_to_error: MpscReceiver<anyhow::Error>, commander: Commander) {
    Rt::spawn_local(async move {
        loop {
            let new_err = receiver_to_error.recv().await.unwrap();

            let back_trace = match new_err.downcast_ref::<HashimError>() {
                Some(_) => None,
                None => {
                    let bt = new_err.backtrace();
                    match bt.status() {
                        BacktraceStatus::Unsupported => None,
                        BacktraceStatus::Disabled => None,
                        BacktraceStatus::Captured => Some(bt.to_string()),
                        _ => unreachable!(),
                    }
                }
            };

            let info = Observe::ErrorArrived {
                name: new_err.to_string(),
                time_unix_ms: Ti::now_as_unix_milliseconds(),
                back_trace,
            };

            commander.send(ProcessId::default(), Message::Observe(info));
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

pub trait GlobalModel {}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct LocalModel {
    pub is_expand_all: bool,
    pub errors: ErrorList,
}

#[derive(Debug, Clone)]
pub enum LocalChange {
    IsExpandAll(bool),
    Errors(ErrorList),
}

#[derive(Debug, Clone)]
pub enum GlobalChange {}

#[derive(Debug, Clone)]
pub enum Effect {}

#[derive(Debug, Clone)]
pub enum Intent {
    DeleteAll,
    DeleteOne(usize),
    ExpandOrCollapseAll,
    ExpandOrCollapseOne(usize),
}

#[derive(Debug, Clone)]
pub enum Observe {
    ErrorArrived {
        name: String,
        time_unix_ms: u64,
        back_trace: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    Intent(Intent),
    Observe(Observe),
}

impl MessageTrait for Message {}

pub fn update(
    msg: Message,
    local_model: &LocalModel,
    _: &impl GlobalModel,
) -> (Vec<LocalChange>, Vec<GlobalChange>, Vec<Effect>) {
    match msg {
        Message::Intent(intent) => match intent {
            Intent::DeleteOne(i) => {
                let mut err = local_model.errors.clone();
                err.0.remove(i);
                (vec![LocalChange::Errors(err)], vec![], vec![])
            }
            Intent::DeleteAll => (
                vec![LocalChange::Errors(ErrorList::default())],
                vec![],
                vec![],
            ),
            Intent::ExpandOrCollapseAll => {
                let a = local_model.is_expand_all.clone();
                (vec![LocalChange::IsExpandAll(a ^ true)], vec![], vec![])
            }
            Intent::ExpandOrCollapseOne(i) => {
                let mut err = local_model.errors.clone();
                let Some(entry) = err.0.get_mut(i) else {
                    return (vec![], vec![], vec![]);
                };
                entry.is_expand ^= true;
                (vec![LocalChange::Errors(err)], vec![], vec![])
            }
        },
        Message::Observe(observe) => match observe {
            Observe::ErrorArrived {
                name,
                time_unix_ms,
                back_trace,
            } => {
                let mut errors = local_model.errors.clone();
                match errors.0.iter_mut().find(|e| e.name == name) {
                    Some(entry) => {
                        if entry.back_trace.is_none() {
                            entry.back_trace = back_trace;
                        }
                        entry.number_of_errors = entry.number_of_errors.saturating_add(1);
                        entry.time_unix_ms = time_unix_ms;
                    }
                    None => {
                        errors.0.push(ErrorInfo {
                            name,
                            number_of_errors: 1,
                            time_unix_ms,
                            back_trace,
                            is_expand: false,
                        });
                    }
                }
                errors.0.sort_by_key(|e| e.time_unix_ms);
                (vec![LocalChange::Errors(errors)], vec![], vec![])
            }
        },
    }
}

pub fn local_apply(msg: LocalChange, local_model: &mut LocalModel) {
    match msg {
        LocalChange::IsExpandAll(i) => local_model.is_expand_all = i,
        LocalChange::Errors(i) => local_model.errors = i,
    }
}

pub fn global_apply(msg: GlobalChange, _: &mut impl GlobalModel) {
    match msg {}
}

pub async fn effect(_: Effect, _: ProcessId, _: UiContext) -> Result<()> {
    Ok(())
}
