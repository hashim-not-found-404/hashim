use crate::utility::components;
use crate::utility::tools;
use dioxus::prelude::*;
use my_core::accounting_client::client_domain::ui_model;
use my_core::accounting_client::client_domain::ui_model::HashimSignal;

#[component]
pub(crate) fn CreateAccount() -> Element {
    use_effect(move || {
        tools::send(ui_model::Message::CreateAccount(ui_model::CreateAccount::Subscribe));
    });

    let local_state = &tools::MODEL.page_create_account;

    let consent_callback = move |consent: ui_model::UserConsent| {
        tools::send(ui_model::Message::CreateAccount(ui_model::CreateAccount::Consent(consent)));
    };

    rsx! {
        div {
            components::Dialog {
                consent_callback,
                operation_name: "create account",
                show_dialog: local_state.show_dialog.clone(),
            }

            input {
                placeholder: "Account Name",
                oninput: move |event| {
                    tools::send(
                        ui_model::Message::CreateAccount(
                            ui_model::CreateAccount::AccountName(event.value()),
                        ),
                    );
                },
                value: local_state.account_name.read(),
            }
            label { {local_state.account_name_error.read().unwrap_or_default()} }

            div {
                label { "Is Debit" }
                input {
                    r#type: "checkbox",
                    checked: local_state.is_debit.read(),
                    onchange: move |event| {
                        tools::send(
                            ui_model::Message::CreateAccount(
                                ui_model::CreateAccount::IsDebit(event.value().parse().unwrap_or(false)),
                            ),
                        );
                    },
                }
            }

            div {
                label { "Is Permanent Account" }
                input {
                    r#type: "checkbox",
                    checked: local_state.is_permanent_account.read(),
                    onchange: move |event| {
                        tools::send(
                            ui_model::Message::CreateAccount(
                                ui_model::CreateAccount::IsPermanentAccount(
                                    event.value().parse().unwrap_or(false),
                                ),
                            ),
                        );
                    },
                }
            }

            input {
                placeholder: "Notes (optional)",
                oninput: move |event| {
                    tools::send(
                        ui_model::Message::CreateAccount(
                            ui_model::CreateAccount::Notes(event.value()),
                        ),
                    );
                },
                value: local_state.notes.read(),
            }

            input {
                placeholder: "Unit of Measurement (e.g., kg, pcs)",
                oninput: move |event| {
                    tools::send(
                        ui_model::Message::CreateAccount(
                            ui_model::CreateAccount::UnitOfMeasurementOfQuantity(event.value()),
                        ),
                    );
                },
                value: local_state.unit_of_measurement_of_quantity.read(),
            }

            button {
                onclick: move |_| {
                    tools::send(ui_model::Message::CreateAccount(ui_model::CreateAccount::Submit));
                },
                "Create Account"
            }

            button {
                onclick: move |_| {
                    tools::send(ui_model::Message::CreateAccount(ui_model::CreateAccount::Clean));
                },
                "clean"
            }
        }
    }
}
