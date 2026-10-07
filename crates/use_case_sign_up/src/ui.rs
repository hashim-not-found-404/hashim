use crate::client::Intent;
use crate::client::LocalModel;
use crate::domain::UserIdError;
use crate::domain::UserNameError;
use dioxus::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use utility::process_manager::UserConsent;
use utility_ui::components::DialogComponent;
use utility_ui::components::PasswordInput;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;
use utility_ui::my_signal::MySignal;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TypeLocalModel {
    show_dialog: MySignal<Dialog>,
    error_user_id: MySignal<Option<UserIdError>>,
    error_user_name: MySignal<Option<UserNameError>>,
}

impl LocalModel for TypeLocalModel {
    fn show_dialog(&self) -> impl HashimSignal<Dialog> {
        self.show_dialog.clone()
    }

    fn error_user_id(&self) -> impl HashimSignal<Option<UserIdError>> {
        self.error_user_id.clone()
    }

    fn error_user_name(&self) -> impl HashimSignal<Option<UserNameError>> {
        self.error_user_name.clone()
    }
}

#[component]
pub fn Component(
    sender: EventHandler<Intent>,
    on_back: EventHandler<()>,
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
                    sender(Intent::Consent(UserConsent::DontWaitForServerResponse));
                },
                wait_for_server_response: move || {
                    sender(Intent::Consent(UserConsent::WaitForServerResponse));
                },
                cancel_operation: move || {
                    sender(Intent::Consent(UserConsent::CancelOperation));
                },
                operation_name: "sign up",
                show_dialog,
            }

            input {
                placeholder: "Name (Optional)",
                oninput: move |event| {
                    sender(Intent::UserName(event.value()));
                },
                value: user_name.unwrap_or_default(),
            }
            if let Some(error) = error_user_name {
                label { {error} }
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
                disabled: false,
                password_callback: move |p| {
                    sender(Intent::Password(p));
                },
                password,
            }

            button {
                onclick: move |_| {
                    sender(Intent::Submit);
                },
                "Sign Up"
            }
            button {
                onclick: move |_| {
                    on_back(());
                },
                "Back"
            }
        }
    }
}
