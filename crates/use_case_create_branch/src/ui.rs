use crate::client::Intent;
use crate::client::LocalModel;
use crate::domain::BranchNameError;
use crate::domain::LocationError;
use dioxus::prelude::*;
use kernel::new_types::CompanyUuid;
use kernel::types::Currency;
use kernel::types::Location;
use serde::Deserialize;
use serde::Serialize;
use std::str::FromStr;
use utility::process_manager::ProcessId;
use utility::process_manager::UserConsent;
use utility_ui::components::DialogComponent;
use utility_ui::components::ListInput;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;
use utility_ui::my_signal::MySignal;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TypeLocalModel {
    is_loading: MySignal<bool>,
    show_dialog: MySignal<Dialog>,
    branch_name: MySignal<String>,
    currency: MySignal<Currency>,
    location: MySignal<Location>,
    branch_name_error: MySignal<Option<BranchNameError>>,
    location_error: MySignal<Option<LocationError>>,
    list_of_companies_to_display: MySignal<Vec<(CompanyUuid, String)>>,
    selected_company_name: MySignal<String>,
    company_name_error: MySignal<String>,
    selected_company_uuid: MySignal<Option<CompanyUuid>>,
}

impl LocalModel for TypeLocalModel {
    fn show_dialog(&self) -> impl HashimSignal<Dialog> {
        self.show_dialog.clone()
    }

    fn is_loading(&self) -> impl HashimSignal<bool> {
        self.is_loading.clone()
    }

    fn list_of_companies_to_display(&self) -> impl HashimSignal<Vec<(CompanyUuid, String)>> {
        self.list_of_companies_to_display.clone()
    }

    fn company_name_error(&self) -> impl HashimSignal<String> {
        self.company_name_error.clone()
    }

    fn selected_company_name(&self) -> impl HashimSignal<String> {
        self.selected_company_name.clone()
    }

    fn selected_company_uuid(&self) -> impl HashimSignal<Option<CompanyUuid>> {
        self.selected_company_uuid.clone()
    }

    fn branch_name(&self) -> impl HashimSignal<String> {
        self.branch_name.clone()
    }

    fn currency(&self) -> impl HashimSignal<Currency> {
        self.currency.clone()
    }

    fn location(&self) -> impl HashimSignal<Location> {
        self.location.clone()
    }

    fn branch_name_error(&self) -> impl HashimSignal<Option<BranchNameError>> {
        self.branch_name_error.clone()
    }

    fn location_error(&self) -> impl HashimSignal<Option<LocationError>> {
        self.location_error.clone()
    }
}

#[component]
pub fn Component(
    sender: EventHandler<Intent>,
    show_dialog: Dialog,
    is_loading: bool,
    selected_company_name: String,
    company_name_error: String,
    selected_company_uuid: String,
    branch_name: String,
    currency: Currency,
    location: Location,
    branch_name_error: Option<String>,
    location_error: Option<String>,
    list_of_company_name_and_uuid: Vec<(CompanyUuid, String)>,
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
                operation_name: "create branch",
                show_dialog,
            }

            ListInput {
                placeholder: "type company name or use default",
                selected_item: selected_company_name,
                on_input: move |a| sender(Intent::CompanyName(a)),
                on_select: move |a| sender(Intent::SelectedCompany(a)),
                list: list_of_company_name_and_uuid,
                row_renderer: Callback::new(move |a: (CompanyUuid, String)| {
                    rsx! {
                        button { "{a.1}" }
                    }
                }),
            }
            label { {company_name_error} }
            label { {selected_company_uuid} }
            input {
                placeholder: "Branch Name",
                oninput: move |event| {
                    sender(Intent::BranchName(event.value()));
                },
                value: branch_name,
            }
            if let Some(err) = branch_name_error {
                label { {err} }
            }

            select {
                value: currency.as_str(),
                onchange: move |event| {
                    let c = Currency::from_str(&event.value()).unwrap_or_default();
                    sender(Intent::Currency(c));
                },
                option { value: "USD", "USD" }
                option { value: "IQD", "IQD" }
            }

            input {
                placeholder: "Latitude",
                oninput: move |event| {
                    sender(Intent::Latitude(event.value().parse().unwrap_or_default()));
                },
                value: "{location.latitude}",
            }
            input {
                placeholder: "Longitude",
                oninput: move |event| {
                    sender(Intent::Longitude(event.value().parse().unwrap_or_default()));
                },
                value: "{location.longitude}",
            }
            if let Some(err) = location_error {
                label { {err} }
            }

            button {
                disabled: is_loading,
                onclick: move |_| {
                    sender(Intent::Submit);
                },
                "Create Branch"
            }
            button {
                onclick: move |_| {
                    sender(Intent::Clean);
                },
                "Clean"
            }
        }
    }
}
