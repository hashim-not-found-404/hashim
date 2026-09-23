use crate::client::ErrorList;
use crate::client::LocalModel;
use crate::client::Message;
use chrono::DateTime;
use dioxus::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use utility_ui::domain::HashimSignal;
use utility_ui::my_signal::MySignal;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TypeLocalModel {
    is_expand_all: MySignal<bool>,
    errors: MySignal<ErrorList>,
}

impl LocalModel for TypeLocalModel {
    fn is_expand_all(&self) -> impl HashimSignal<bool> {
        self.is_expand_all.clone()
    }

    fn errors(&self) -> impl HashimSignal<ErrorList> {
        self.errors.clone()
    }
}

#[component]
pub fn Component(sender: EventHandler<Message>, is_expand_all: bool, errors: ErrorList) -> Element {
    if errors.0.is_empty() {
        return rsx!();
    }

    // Collapsed: show just a small badge so the user can bring the panel back.
    if !is_expand_all {
        let count = errors.0.len();
        return rsx! {
            button { onclick: move |_| sender(Message::ExpandOrCollapseAll),
                "⚠ {count} errors"
            }
        };
    }

    // Expanded: full panel.
    rsx! {
        div {
            div {
                button { onclick: move |_| sender(Message::ExpandOrCollapseAll),
                    "Collapse All"
                }
                button { onclick: move |_| sender(Message::DeleteAll), "Delete All" }
            }

            for (index, error) in errors.0.iter().enumerate() {
                {
                    let name = error.name.clone();
                    let count = error.number_of_errors;
                    let time = format_unix_ms(error.time_unix_ms);
                    let back_trace = error.back_trace.clone();
                    let is_error_expanded = error.is_expand;

                    rsx! {
                        div {
                            div {
                                label { "{count} " }
                                label { "{name} " }
                                label { "{time}" }
                                button { onclick: move |_| { sender(Message::ExpandOrCollapseOne(index)) },
                                    if is_error_expanded {
                                        "Hide"
                                    } else {
                                        "Show"
                                    }
                                }
                                button { onclick: move |_| { sender(Message::DeleteOne(index)) },
                                    "X"
                                }
                            }
                            if is_error_expanded {
                                if let Some(bt) = back_trace {
                                    pre { "{bt}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn format_unix_ms(ms: u64) -> String {
    DateTime::from_timestamp_millis(ms as i64)
        .unwrap_or_default()
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string()
}
