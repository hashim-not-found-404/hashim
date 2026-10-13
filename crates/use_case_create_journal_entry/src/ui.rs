use crate::client::Intent;
use dioxus::prelude::*;
use utility::ui_orchestration::UserConsent;
use utility_ui::components::DialogComponent;
use utility_ui::domain::Dialog;

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
