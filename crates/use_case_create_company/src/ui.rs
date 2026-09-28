use crate::client::LocalModel;
use crate::client::Message;
use dioxus::prelude::*;
use kernel::types::Currency;
use serde::Deserialize;
use serde::Serialize;
use std::str::FromStr;
use utility::process_manager::ProcessId;
use utility::process_manager::UserConsent;
use utility_ui::components::DialogComponent;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;
use utility_ui::my_signal::MySignal;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TypeLocalModel {
    process_id: MySignal<Option<ProcessId>>,
    is_loading: MySignal<bool>,
    show_dialog: MySignal<Dialog>,
    company_name: MySignal<String>,
    currency: MySignal<Currency>,
    company_name_error: MySignal<Option<String>>,
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

    fn company_name(&self) -> impl HashimSignal<String> {
        self.company_name.clone()
    }

    fn currency(&self) -> impl HashimSignal<Currency> {
        self.currency.clone()
    }

    fn company_name_error(&self) -> impl HashimSignal<Option<String>> {
        self.company_name_error.clone()
    }
}

#[component]
pub fn Component(
    sender: EventHandler<Message>,
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
                    sender(Message::Consent(UserConsent::DontWaitForServerResponse));
                },
                wait_for_server_response: move || {
                    sender(Message::Consent(UserConsent::WaitForServerResponse));
                },
                cancel_operation: move || {
                    sender(Message::Consent(UserConsent::CancelOperation));
                },
                operation_name: "create company",
                show_dialog,
            }

            input {
                placeholder: "Company Name",
                oninput: move |event| {
                    sender(Message::CompanyName(event.value()));
                },
                value: company_name,
            }
            if let Some(err) = company_name_error {
                label { {err} }
            }

            select {
                value: currency.as_str(),
                onchange: move |event| {
                    let c = Currency::from_str(&event.value()).unwrap_or_default();
                    sender(Message::Currency(c));
                },
                option { value: "USD", "USD" }
                option { value: "IQD", "IQD" }
            }

            button {
                disabled: is_loading,
                onclick: move |_| {
                    sender(Message::Submit);
                },
                "Create Company"
            }
            button {
                onclick: move |_| {
                    sender(Message::Clean);
                },
                "Clean"
            }
        }
    }
}
