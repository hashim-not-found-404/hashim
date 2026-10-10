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

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TypeLocalModel {
    show_dialog: MySignal<Dialog>,
    is_loading: MySignal<bool>,
    shared_entry_id: MySignal<String>,
    some_account_are_not_inferred: MySignal<bool>,
    error_container_is_empty: MySignal<bool>,
    not_all_entry_inferred: MySignal<bool>,
    double_entries: MySignal<Vec<DoubleEntry>>,
    filtered_list: MySignal<Vec<Account>>,
    list_of_available_account: MySignal<Vec<Account>>,
    async_state: MySignal<AsyncState>,
}

impl LocalModel for TypeLocalModel {
    fn show_dialog(&self) -> impl HashimSignal<Dialog> {
        self.show_dialog.clone()
    }
    fn is_loading(&self) -> impl HashimSignal<bool> {
        self.is_loading.clone()
    }
    fn shared_entry_id(&self) -> impl HashimSignal<String> {
        self.shared_entry_id.clone()
    }
    fn some_account_are_not_inferred(&self) -> impl HashimSignal<bool> {
        self.some_account_are_not_inferred.clone()
    }
    fn error_container_is_empty(&self) -> impl HashimSignal<bool> {
        self.error_container_is_empty.clone()
    }
    fn not_all_entry_inferred(&self) -> impl HashimSignal<bool> {
        self.not_all_entry_inferred.clone()
    }
    fn double_entries(&self) -> impl HashimSignal<Vec<DoubleEntry>> {
        self.double_entries.clone()
    }
    fn filtered_list(&self) -> impl HashimSignal<Vec<Account>> {
        self.filtered_list.clone()
    }
    fn list_of_available_account(&self) -> impl HashimSignal<Vec<Account>> {
        self.list_of_available_account.clone()
    }
}

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
