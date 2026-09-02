use crate::utility::components;
use crate::utility::tools;
use accounting_engine::accounting_stuff;
use dioxus::prelude::*;
use my_core::client::utility::ui_model;
use my_core::client::utility::ui_model::HashimSignal;
use std::str::FromStr;

#[component]
pub(crate) fn CreateAccountForBranch() -> Element {
    use_effect(move || {
        tools::send(ui_model::Message::CreateAccountForBranch(
            ui_model::CreateAccountForBranch::Subscribe,
        ));
    });

    use_drop(move || {
        tools::send(ui_model::Message::CreateAccountForBranch(
            ui_model::CreateAccountForBranch::UnSubscribe,
        ));
    });

    let model = &tools::MODEL;
    let local_state = &model.page_create_account_for_branch;

    let consent_callback = move |consent: ui_model::UserConsent| {
        tools::send(ui_model::Message::CreateAccountForBranch(
            ui_model::CreateAccountForBranch::Consent(consent),
        ));
    };

    rsx! {
        div {
            components::Dialog {
                consent_callback,
                operation_name: "create account for branch",
                show_dialog: local_state.show_dialog.clone(),
            }

            div {
                input {
                    placeholder: "Account Name (type to search)",
                    oninput: move |event| {
                        tools::send(
                            ui_model::Message::CreateAccountForBranch(
                                ui_model::CreateAccountForBranch::AccountName(event.value()),
                            ),
                        );
                    },
                    value: local_state.account_name.read(),
                }
                div {
                    for account in local_state.filtered_list.read() {
                        div {
                            onclick: move |_| {
                                tools::send(
                                    ui_model::Message::CreateAccountForBranch(
                                        ui_model::CreateAccountForBranch::AccountName(
                                            account.account_name.clone(),
                                        ),
                                    ),
                                );
                            },
                            "{account.account_name}"
                        }
                    }
                }
            }

            select {
                value: local_state.outflow_type.read().as_str(),
                onchange: move |event| {
                    let value = event.value();
                    let outflow_type = accounting_stuff::OutFlowType::from_str(&value)
                        .unwrap_or_default();
                    tools::send(
                        ui_model::Message::CreateAccountForBranch(
                            ui_model::CreateAccountForBranch::OutflowType(outflow_type),
                        ),
                    );
                },
                option { value: "None", "None" }
                option { value: "QuantityEqualAmount", "QuantityEqualAmount" }
                option { value: "QuantityEqualZero", "QuantityEqualZero" }
                option { value: "Wac", "Wac" }
                option { value: "Fifo", "Fifo" }
                option { value: "Lifo", "Lifo" }
                option { value: "Hifo", "Hifo" }
                option { value: "Lofo", "Lofo" }
            }

            select {
                value: local_state.inflow_type.read().as_str(),
                onchange: move |event| {
                    let value = event.value();
                    let inflow_type = accounting_stuff::InFlowType::from_str(&value)
                        .unwrap_or_default();
                    tools::send(
                        ui_model::Message::CreateAccountForBranch(
                            ui_model::CreateAccountForBranch::InflowType(inflow_type),
                        ),
                    );
                },
                option { value: "None", "None" }
                option { value: "QuantityEqualAmount", "QuantityEqualAmount" }
                option { value: "QuantityEqualZero", "QuantityEqualZero" }
                option { value: "Wac", "Wac" }
            }

            button {
                disabled: local_state.is_loading.read(),
                onclick: move |_| {
                    tools::send(
                        ui_model::Message::CreateAccountForBranch(
                            ui_model::CreateAccountForBranch::Submit,
                        ),
                    );
                },
                "Create Account for Branch"
            }

            button {
                onclick: move |_| {
                    tools::send(
                        ui_model::Message::CreateAccountForBranch(
                            ui_model::CreateAccountForBranch::Clean,
                        ),
                    );
                },
                "Clear"
            }
        }
    }
}
