use crate::client::Message;
use dioxus::prelude::*;
use utility::process_manager::UserConsent;
use utility_ui::components::DialogComponent;
use utility_ui::domain::Dialog;

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
                    show_dialog: show_dialog,
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
