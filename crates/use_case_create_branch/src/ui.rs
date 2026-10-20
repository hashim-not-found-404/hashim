use crate::client::Intent;
use dioxus::prelude::*;
use kernel::new_types::CompanyUuid;
use kernel::types::Currency;
use kernel::types::Location;
use std::str::FromStr;
use utility::ui_orchestration::UserConsent;
use utility_ui::components::DialogComponent;
use utility_ui::components::ListInput;
use utility_ui::domain::Dialog;

#[component]
pub fn Component(
    sender: EventHandler<Intent>,
    show_dialog: Dialog,
    is_loading: bool,
    company_name: String,
    company_name_error: Option<String>,
    resolved_company_uuid: Option<String>,
    branch_name: String,
    currency: Currency,
    location: Location,
    branch_name_error: Option<String>,
    location_error: Option<String>,
    list_of_company_name_and_uuid: Vec<(CompanyUuid, String)>,
) -> Element {
    use_effect(move || {
        sender(Intent::Subscribe);
    });

    use_drop(move || {
        sender(Intent::UnSubscribe);
    });

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
                disabled: is_loading,
                placeholder: "type company name or use default",
                selected_item: company_name,
                on_input: move |a| sender(Intent::CompanyName(a)),
                on_select: move |a| sender(Intent::SelectedCompany(a)),
                list: list_of_company_name_and_uuid,
                row_renderer: Callback::new(move |a: (CompanyUuid, String)| {
                    rsx! {
                        button { disabled: is_loading, "{a.1}" }
                    }
                }),
            }
            if let Some(err) = company_name_error {
                label { {err} }
            }

            if let Some(uuid) = resolved_company_uuid {
                label { "{uuid}" }
            }

            input {
                disabled: is_loading,
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
                disabled: is_loading,
                value: currency.as_str(),
                onchange: move |event| {
                    let c = Currency::from_str(&event.value()).unwrap_or_default();
                    sender(Intent::Currency(c));
                },
                option { value: "USD", "USD" }
                option { value: "IQD", "IQD" }
            }

            input {
                disabled: is_loading,
                placeholder: "Latitude",
                oninput: move |event| {
                    sender(Intent::Latitude(event.value().parse().unwrap_or_default()));
                },
                value: "{location.latitude}",
            }
            input {
                disabled: is_loading,
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
                "Submit"
            }
        }
    }
}
