use crate::client::LocalModel;
use crate::client::Message;
use dioxus::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use utility_ui::domain::HashimSignal;
use utility_ui::my_signal::MySignal;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TypeLocalModel {
    errors: MySignal<Vec<String>>,
}

impl LocalModel for TypeLocalModel {
    fn errors(&self) -> impl HashimSignal<Vec<String>> {
        self.errors.clone()
    }
}

#[component]
pub fn ErrorStack(close_error_callback: EventHandler<Message>, errors: Vec<String>) -> Element {
    if errors.is_empty() {
        return rsx!();
    }

    rsx! {
        div {
            for (index, error) in errors.into_iter().enumerate() {
                div {
                    label { {error} }
                    button {
                        onclick: move |_| close_error_callback(Message::CloseError(index)),
                        "X"
                    }
                }
            }
        }
    }
}
