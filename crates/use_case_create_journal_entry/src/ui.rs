use crate::client::Account;
use crate::client::AsyncState;
use crate::client::DoubleEntry;
use crate::client::Intent;
use crate::client::LocalModel;
use dioxus::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use utility::ui_orchestration::UserConsent;
use utility_ui::components::DialogComponent;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;
use utility_ui::my_signal::MySignal;

#[component]
pub fn Component(sender: EventHandler<Intent>, show_dialog: Dialog, is_loading: bool) -> Element {
    use_effect(move || {
        sender(Intent::Subscribe);
    });

    use_drop(move || {
        sender(Intent::UnSubscribe);
    });

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
                operation_name: "create journal entry",
                show_dialog,
            }

            // The full double-entry editor is intentionally a stub for now.
            // The reducer already supports Add/Remove/Update of double and
            // single entries; the UI simply has not been fleshed out yet.
            button {
                disabled: is_loading,
                onclick: move |_| {
                    sender(Intent::Submit);
                },
                "Submit"
            }
            button {
                disabled: is_loading,
                onclick: move |_| {
                    sender(Intent::Clean);
                },
                "Clean"
            }
        }
    }
}
