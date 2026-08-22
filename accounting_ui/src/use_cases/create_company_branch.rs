use crate::utility::components;
use crate::utility::tools;
use dioxus::prelude::*;
use my_core::accounting_client::client_domain::ui_model;
use my_core::accounting_client::client_domain::ui_model::HashimSignal;
use my_core::accounting_domain::utility::types::Currency;
use std::str::FromStr;

#[component]
pub(crate) fn CreateCompanyBranch() -> Element {
    let local_state = &tools::MODEL.page_create_company_branch;

    let consent_callback = move |consent: ui_model::UserConsent| {
        tools::send(ui_model::Message::CreateCompanyBranch(
            ui_model::CreateCompanyBranch::Consent(consent),
        ));
    };

    rsx! {
        div {
            components::Dialog {
                consent_callback,
                operation_name: "create company branch",
                show_dialog: local_state.show_dialog.clone(),
            }
            input {
                placeholder: "Branch Name",
                oninput: move |event| {
                    tools::send(
                        ui_model::Message::CreateCompanyBranch(
                            ui_model::CreateCompanyBranch::Name(event.value()),
                        ),
                    );
                },
                value: local_state.branch_name.read(),
            }
            select {
                value: local_state.currency.read().as_str(),
                onchange: move |event| {
                    let currency = Currency::from_str(&event.value()).unwrap_or_default();
                    tools::send(
                        ui_model::Message::CreateCompanyBranch(
                            ui_model::CreateCompanyBranch::Currency(currency),
                        ),
                    );
                },
                option { value: "USD", "USD" }
                option { value: "IQD", "IQD" }
            }
            button {
                onclick: move |_| {
                    tools::send(
                        ui_model::Message::CreateCompanyBranch(ui_model::CreateCompanyBranch::Submit),
                    );
                },
                "Create"
            }
            button {
                onclick: move |_| {
                    tools::send(
                        ui_model::Message::CreateCompanyBranch(ui_model::CreateCompanyBranch::Close),
                    );
                },
                "X"
            }
        }
    }
}
