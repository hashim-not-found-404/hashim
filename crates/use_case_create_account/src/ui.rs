use crate::client::LocalModel;
use crate::client::Message;
use dioxus::prelude::*;
use utility::process_manager::UserConsent;
use utility::ui_effect::Commander;
use utility_ui::components::DialogComponent;
use utility_ui::domain::HashimSignal;

#[component]
pub fn CreateAccount<LM>(commander: Commander, model: LM) -> Element
where
    LM: LocalModel,
{
    let local_state = model;

    let commander1 = commander.clone();
    use_effect(move || {
        commander1.send(Message::Subscribe);
    });

    let commander1 = commander.clone();
    let dont_wait_for_server_response = move || {
        commander1.send(Message::Consent(UserConsent::DontWaitForServerResponse));
    };

    let commander1 = commander.clone();
    let wait_for_server_response = move || {
        commander1.send(Message::Consent(UserConsent::WaitForServerResponse));
    };

    let commander1 = commander.clone();
    let cancel_operation = move || {
        commander1.send(Message::Consent(UserConsent::CancelOperation));
    };

    let commander1 = commander.clone();
    let commander2 = commander.clone();
    let commander3 = commander.clone();
    let commander4 = commander.clone();
    let commander5 = commander.clone();
    let commander6 = commander.clone();

    rsx! {
        div {
            DialogComponent {
                dont_wait_for_server_response,
                wait_for_server_response,
                cancel_operation,
                operation_name: "create account",
                show_dialog: local_state.show_dialog().read(),
            }
            input {
                placeholder: "Account Name",
                oninput: move |event| {
                    commander1.send(Message::AccountName(event.value()));
                },
                value: local_state.account_name().read(),
            }
            label { {local_state.account_name_error().read().unwrap_or_default()} }
            div {
                label { "Is Debit" }
                input {
                    r#type: "checkbox",
                    checked: local_state.is_debit().read(),
                    onchange: move |event| {
                        commander2.send(Message::IsDebit(event.value().parse().unwrap_or_default()));
                    },
                }
            }
            div {
                label { "Is Permanent Account" }
                input {
                    r#type: "checkbox",
                    checked: local_state.is_permanent_account().read(),
                    onchange: move |event| {
                        commander3
                            .send(Message::IsPermanentAccount(event.value().parse().unwrap_or_default()));
                    },
                }
            }
            input {
                placeholder: "Notes (optional)",
                oninput: move |event| {
                    commander4.send(Message::Notes(event.value()));
                },
                value: local_state.notes().read(),
            }
            input {
                placeholder: "Unit of Measurement (e.g., kg, pcs)",
                oninput: move |event| {
                    commander5.send(Message::UnitOfMeasurementOfQuantity(event.value()));
                },
                value: local_state.unit_of_measurement_of_quantity().read(),
            }
            button {
                onclick: move |_| {
                    commander6.send(Message::Submit);
                },
                "Create Account"
            }
            button {
                onclick: move |_| {
                    commander.send(Message::Clean);
                },
                "clean"
            }
        }
    }
}
