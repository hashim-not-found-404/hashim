use crate::client::LocalModel;
use crate::client::Message;
use dioxus::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use utility::process_manager::ProcessId;
use utility::process_manager::UserConsent;
use utility_ui::components::DialogComponent;
use utility_ui::components::PasswordInput;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;
use utility_ui::my_signal::MySignal;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TypeLocalModel {
    process_id: MySignal<Option<ProcessId>>,
    show_dialog: MySignal<Dialog>,
    error_user_id: MySignal<Option<String>>,
    error_user_name: MySignal<Option<String>>,
}

impl LocalModel for TypeLocalModel {
    fn process_id(&self) -> impl HashimSignal<Option<ProcessId>> {
        self.process_id.clone()
    }

    fn show_dialog(&self) -> impl HashimSignal<Dialog> {
        self.show_dialog.clone()
    }

    fn error_user_id(&self) -> impl HashimSignal<Option<String>> {
        self.error_user_id.clone()
    }

    fn error_user_name(&self) -> impl HashimSignal<Option<String>> {
        self.error_user_name.clone()
    }
}

#[component]
pub fn Component(
    sender: EventHandler<Message>,
    show_dialog: Dialog,
    user_id: String,
    user_name: Option<String>,
    password: String,
    error_user_id: Option<String>,
    error_user_name: Option<String>,
) -> Element {
    rsx! {
        div {
            DialogComponent {
                dont_wait_for_server_response: move || {
                    sender(Message::Consent(UserConsent::DontWaitForServerResponse));
                },
                wait_for_server_response: move || {
                    sender(Message::Consent(UserConsent::WaitForServerResponse));
                },
                cancel_operation: move || {
                    sender(Message::Consent(UserConsent::CancelOperation));
                },
                operation_name: "sign up",
                show_dialog,
            }

            input {
                placeholder: "Name (Optional)",
                oninput: move |event| {
                    sender(Message::UserName(event.value()));
                },
                value: user_name.unwrap_or_default(),
            }
            if let Some(error) = error_user_name {
                label { {error} }
            }

            input {
                placeholder: "User Id",
                oninput: move |event| {
                    sender(Message::UserId(event.value()));
                },
                value: user_id,
            }
            if let Some(error) = error_user_id {
                label { {error} }
            }

            PasswordInput {
                password_callback: move |p| {
                    sender(Message::Password(p));
                },
                password,
            }

            button {
                onclick: move |_| {
                    sender(Message::Submit);
                },
                "Sign Up"
            }
            button {
                onclick: move |_| {
                    sender(Message::GoToSignIn);
                },
                "Back"
            }
        }
    }
}
