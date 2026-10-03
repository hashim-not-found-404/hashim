use crate::client::Intent;
use crate::client::LocalModel;
use crate::domain::PasswordError;
use crate::domain::UserIdError;
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
    error_user_id: MySignal<Option<UserIdError>>,
    error_password: MySignal<Option<PasswordError>>,
}

impl LocalModel for TypeLocalModel {
    fn show_dialog(&self) -> impl HashimSignal<Dialog> {
        self.show_dialog.clone()
    }

    fn error_user_id(&self) -> impl HashimSignal<Option<UserIdError>> {
        self.error_user_id.clone()
    }

    fn error_password(&self) -> impl HashimSignal<Option<PasswordError>> {
        self.error_password.clone()
    }
}

#[component]
pub fn Component(
    sender: EventHandler<Intent>,
    on_go_to_sign_up: EventHandler<()>,
    show_dialog: Dialog,
    user_id: String,
    password: String,
    error_user_id: Option<String>,
    error_password: Option<String>,
) -> Element {
    rsx! {
        div {
            DialogComponent {
                dont_wait_for_server_response: move || {
                    sender(Intent::Consent(UserConsent::DontWaitForServerResponse));
                },
                wait_for_server_response: move || {
                    sender(Intent::Consent(UserConsent::WaitForServerResponse));
                },
                cancel_operation: move || {
                    sender(Intent::Consent(UserConsent::CancelOperation));
                },
                operation_name: "sign in",
                show_dialog,
            }

            input {
                placeholder: "User Id",
                oninput: move |event| {
                    sender(Intent::UserId(event.value()));
                },
                value: user_id,
            }
            if let Some(error) = error_user_id {
                label { {error} }
            }

            PasswordInput {
                password_callback: move |p| {
                    sender(Intent::Password(p));
                },
                password,
            }
            if let Some(error) = error_password {
                label { {error} }
            }

            button {
                onclick: move |_| {
                    sender(Intent::Submit);
                },
                "Sign In"
            }
            button {
                onclick: move |_| {
                    on_go_to_sign_up(());
                },
                "Sign Up"
            }
        }
    }
}
