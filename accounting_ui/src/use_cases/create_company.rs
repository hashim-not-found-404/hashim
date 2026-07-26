use crate::utility::tools;
use dioxus::prelude::*;
use my_core::accounting_client::client_domain::ui_model;
use my_core::accounting_client::client_domain::ui_model::HashimSignal;
use my_core::accounting_domain::utility::types;
use std::str::FromStr;

#[component]
pub(crate) fn CreateCompany() -> Element {
    let local_state = &tools::MODEL.page_create_company;

    rsx! {
        div {
            input {
                placeholder: "Company Name",
                oninput: move |event| {
                    tools::send(
                        ui_model::Message::CreateCompany(
                            ui_model::CreateCompany::Name(event.value()),
                        ),
                    );
                },
                value: local_state.company_name.read(),
            }
            select {
                value: local_state.currency.read().as_str(),
                onchange: move |event| {
                    let currency = types::Currency::from_str(&event.value()).unwrap_or_default();
                    tools::send(
                        ui_model::Message::CreateCompany(ui_model::CreateCompany::Currency(currency)),
                    );
                },
                option { value: "USD", "USD" }
                option { value: "IQD", "IQD" }
            }
            button {
                onclick: move |_| {
                    tools::send(ui_model::Message::CreateCompany(ui_model::CreateCompany::Submit));
                },
                "Create"
            }
            button {
                onclick: move |_| {
                    tools::send(ui_model::Message::CreateCompany(ui_model::CreateCompany::Close));
                },
                "X"
            }
        }
    }
}
