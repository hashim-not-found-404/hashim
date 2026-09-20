use crate::client::LocalModel;
use crate::client::Message;
use dioxus::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use utility::process_manager::ProcessId;
use utility::process_manager::UserConsent;
use utility_ui::components::DialogComponent;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;
use utility_ui::my_signal::MySignal;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TypeLocalModel {
    pub process_id: MySignal<Option<ProcessId>>,
    pub is_loading: MySignal<bool>,
    pub show_dialog: MySignal<Dialog>,
    pub is_debit: MySignal<bool>,
    pub is_permanent_account: MySignal<bool>,
    pub account_name: MySignal<String>,
    pub notes: MySignal<String>,
    pub unit_of_measurement_of_quantity: MySignal<String>,
    pub account_name_error: MySignal<Option<String>>,
}

impl LocalModel for TypeLocalModel {
    fn process_id(&self) -> impl HashimSignal<Option<ProcessId>> {
        self.process_id.clone()
    }

    fn show_dialog(&self) -> impl HashimSignal<Dialog> {
        self.show_dialog.clone()
    }

    fn is_loading(&self) -> impl HashimSignal<bool> {
        self.is_loading.clone()
    }

    fn is_debit(&self) -> impl HashimSignal<bool> {
        self.is_debit.clone()
    }

    fn is_permanent_account(&self) -> impl HashimSignal<bool> {
        self.is_permanent_account.clone()
    }

    fn account_name(&self) -> impl HashimSignal<String> {
        self.account_name.clone()
    }

    fn notes(&self) -> impl HashimSignal<String> {
        self.notes.clone()
    }

    fn unit_of_measurement_of_quantity(&self) -> impl HashimSignal<String> {
        self.unit_of_measurement_of_quantity.clone()
    }

    fn account_name_error(&self) -> impl HashimSignal<Option<String>> {
        self.account_name_error.clone()
    }
}

#[component]
pub fn CreateAccount(
    sender: EventHandler<Message>,
    show_dialog: Dialog,
    is_loading: bool,
    is_debit: bool,
    is_permanent_account: bool,
    account_name: String,
    notes: String,
    unit_of_measurement_of_quantity: String,
    account_name_error: String,
) -> Element {
    use_effect(move || {
        sender(Message::Subscribe);
    });

    let dont_wait_for_server_response = move || {
        sender(Message::Consent(UserConsent::DontWaitForServerResponse));
    };

    let wait_for_server_response = move || {
        sender(Message::Consent(UserConsent::WaitForServerResponse));
    };

    let cancel_operation = move || {
        sender(Message::Consent(UserConsent::CancelOperation));
    };

    rsx! {
        div {
            DialogComponent {
                dont_wait_for_server_response,
                wait_for_server_response,
                cancel_operation,
                operation_name: "create account",
                show_dialog,
            }
            input {
                placeholder: "Account Name",
                oninput: move |event| {
                    sender(Message::AccountName(event.value()));
                },
                value: account_name,
            }
            label { {account_name_error} }
            div {
                label { "Is Debit" }
                input {
                    r#type: "checkbox",
                    checked: is_debit,
                    onchange: move |event| {
                        sender(Message::IsDebit(event.value().parse().unwrap_or_default()));
                    },
                }
            }
            div {
                label { "Is Permanent Account" }
                input {
                    r#type: "checkbox",
                    checked: is_permanent_account,
                    onchange: move |event| {
                        sender(Message::IsPermanentAccount(event.value().parse().unwrap_or_default()));
                    },
                }
            }
            input {
                placeholder: "Notes (optional)",
                oninput: move |event| {
                    sender(Message::Notes(event.value()));
                },
                value: notes,
            }
            input {
                placeholder: "Unit of Measurement (e.g., kg, pcs)",
                oninput: move |event| {
                    sender(Message::UnitOfMeasurementOfQuantity(event.value()));
                },
                value: unit_of_measurement_of_quantity,
            }
            button {
                onclick: move |_| {
                    sender(Message::Submit);
                },
                "Create Account"
            }
            button {
                onclick: move |_| {
                    sender(Message::Clean);
                },
                "clean"
            }
        }
    }
}
