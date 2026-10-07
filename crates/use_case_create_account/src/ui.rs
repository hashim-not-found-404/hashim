use crate::client::CompanyNameError;
use crate::client::Intent;
use crate::client::LocalModel;
use crate::domain::AccountNameError;
use dioxus::prelude::*;
use kernel::new_types::CompanyUuid;
use serde::Deserialize;
use serde::Serialize;
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
    is_debit: MySignal<bool>,
    is_permanent_account: MySignal<bool>,
    account_name: MySignal<String>,
    unit_of_measurement_of_quantity: MySignal<String>,
    account_name_error: MySignal<Option<AccountNameError>>,
    selected_company_name: MySignal<String>,
    selected_company_uuid: MySignal<Option<CompanyUuid>>,
    list_of_companies_to_display: MySignal<Vec<(CompanyUuid, String)>>,
    company_name_error: MySignal<Option<CompanyNameError>>,
}

impl LocalModel for TypeLocalModel {
    fn show_dialog(&self) -> impl HashimSignal<Dialog> {
        self.show_dialog.clone()
    }

    fn is_loading(&self) -> impl HashimSignal<bool> {
        self.is_loading.clone()
    }

    fn is_debit(&self) -> impl HashimSignal<bool> {
        self.is_debit.clone()
    }

    fn is_permanent_account(&self) -> impl HashimSignal<bool> {
        self.is_permanent_account.clone()
    }

    fn account_name(&self) -> impl HashimSignal<String> {
        self.account_name.clone()
    }

    fn unit_of_measurement_of_quantity(&self) -> impl HashimSignal<String> {
        self.unit_of_measurement_of_quantity.clone()
    }

    fn account_name_error(&self) -> impl HashimSignal<Option<AccountNameError>> {
        self.account_name_error.clone()
    }

    fn selected_company_name(&self) -> impl HashimSignal<String> {
        self.selected_company_name.clone()
    }

    fn selected_company_uuid(&self) -> impl HashimSignal<Option<CompanyUuid>> {
        self.selected_company_uuid.clone()
    }

    fn list_of_companies_to_display(&self) -> impl HashimSignal<Vec<(CompanyUuid, String)>> {
        self.list_of_companies_to_display.clone()
    }

    fn company_name_error(&self) -> impl HashimSignal<Option<CompanyNameError>> {
        self.company_name_error.clone()
    }
}

#[component]
pub fn Component(
    sender: EventHandler<Intent>,
    show_dialog: Dialog,
    is_loading: bool,
    is_debit: bool,
    is_permanent_account: bool,
    account_name: String,
    unit_of_measurement_of_quantity: String,
    account_name_error: Option<String>,
    selected_company_name: String,
    selected_company_uuid: String,
    company_name_error: Option<CompanyNameError>,
    list_of_company_name_and_uuid: Vec<(CompanyUuid, String)>,
) -> Element {
    use_effect(move || {
        sender(Intent::Subscribe);
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
                operation_name: "create account",
                show_dialog,
            }

            ListInput {
                disabled: is_loading,
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

            div {
                if !selected_company_uuid.is_empty() {
                    label { "{selected_company_uuid}" }
                }
            }

            if let Some(err) = company_name_error {
                // Translation hook: replace `format!("{err:?}")` with your
                // translator once it exists, e.g.
                //     label { {translate(err)} }
                label { {format!("{err:?}")} }
            }

            input {
                placeholder: "Account Name",
                oninput: move |event| {
                    sender(Intent::AccountName(event.value()));
                },
                value: account_name,
            }
            if let Some(account_name_error) = account_name_error {
                label { {account_name_error} }
            }

            div {
                label { "Is Debit" }
                input {
                    r#type: "checkbox",
                    checked: is_debit,
                    onchange: move |event| {
                        sender(Intent::IsDebit(event.value().parse().unwrap_or_default()));
                    },
                }
            }

            div {
                label { "Is Permanent Account" }
                input {
                    r#type: "checkbox",
                    checked: is_permanent_account,
                    onchange: move |event| {
                        sender(Intent::IsPermanentAccount(event.value().parse().unwrap_or_default()));
                    },
                }
            }

            input {
                placeholder: "Unit of Measurement (e.g., kg, pcs)",
                oninput: move |event| {
                    sender(Intent::UnitOfMeasurementOfQuantity(event.value()));
                },
                value: unit_of_measurement_of_quantity,
            }

            button {
                disabled: is_loading,
                onclick: move |_| {
                    sender(Intent::Submit);
                },
                "Create Account"
            }
            button {
                onclick: move |_| {
                    sender(Intent::Clean);
                },
                "clean"
            }
        }
    }
}
