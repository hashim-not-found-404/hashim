use crate::client::AsyncState;
use crate::client::Intent;
use crate::client::LocalModel;
use dioxus::prelude::*;
use kernel::types::Currency;
use serde::Deserialize;
use serde::Serialize;
use std::str::FromStr;
use utility::ui_orchestration::UserConsent;
use utility_ui::components::DialogComponent;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;
use utility_ui::my_signal::MySignal;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TypeLocalModel {
    show_dialog: MySignal<Dialog>,
    company_name: MySignal<String>,
    currency: MySignal<Currency>,
    async_state: MySignal<AsyncState>,
}

impl LocalModel for TypeLocalModel {
    fn show_dialog(&self) -> impl HashimSignal<Dialog> {
        self.show_dialog.clone()
    }

    fn company_name(&self) -> impl HashimSignal<String> {
        self.company_name.clone()
    }

    fn currency(&self) -> impl HashimSignal<Currency> {
        self.currency.clone()
    }

    fn async_state(&self) -> impl HashimSignal<AsyncState> {
        self.async_state.clone()
    }
}

#[component]
pub fn Component(
    sender: EventHandler<Intent>,
    show_dialog: Dialog,
    is_loading: bool,
    company_name: String,
    currency: Currency,
    company_name_error: Option<String>,
) -> Element {
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
                operation_name: "create company",
                show_dialog,
            }

            input {
                disabled: is_loading,
                placeholder: "Company Name",
                oninput: move |event| {
                    sender(Intent::CompanyName(event.value()));
                },
                value: company_name,
            }
            if let Some(err) = company_name_error {
                label { {err} }
            }

            select {
                disabled: is_loading,
                value: currency.as_str(),
                onchange: move |event| {
                    let c = Currency::from_str(&event.value()).unwrap_or_default();
                    sender(Intent::Currency(c));
                },
                option { value: "USD", "USD" }
                option { value: "IQD", "IQD" }
            }

            button {
                disabled: is_loading,
                onclick: move |_| {
                    sender(Intent::Submit);
                },
                "Create Company"
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
