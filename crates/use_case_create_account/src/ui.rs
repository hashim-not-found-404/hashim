use crate::client::AsyncState;
use crate::client::Intent;
use crate::client::LocalModel;
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
    show_dialog: MySignal<Dialog>,
    is_debit: MySignal<bool>,
    is_permanent_account: MySignal<bool>,
    account_name: MySignal<String>,
    unit_of_measurement_of_quantity: MySignal<String>,
    selected_company_name: MySignal<String>,
    async_state: MySignal<AsyncState>,
}

impl LocalModel for TypeLocalModel {
    fn show_dialog(&self) -> impl HashimSignal<Dialog> {
        self.show_dialog.clone()
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

    fn selected_company_name(&self) -> impl HashimSignal<String> {
        self.selected_company_name.clone()
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
    is_debit: bool,
    is_permanent_account: bool,
    account_name: String,
    unit_of_measurement_of_quantity: String,
    account_name_error: Option<String>,
    company_name_error: Option<String>,
    selected_company_name: String,
    resolved_company_uuid: Option<String>,
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
                placeholder: "Account Name",
                oninput: move |event| {
                    sender(Intent::AccountName(event.value()));
                },
                value: account_name,
            }
            if let Some(err) = account_name_error {
                label { {err} }
            }

            div {
                label { "Is Debit" }
                input {
                    disabled: is_loading,
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
                    disabled: is_loading,
                    r#type: "checkbox",
                    checked: is_permanent_account,
                    onchange: move |event| {
                        sender(Intent::IsPermanentAccount(event.value().parse().unwrap_or_default()));
                    },
                }
            }

            input {
                disabled: is_loading,
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
                disabled: is_loading,
                onclick: move |_| {
                    sender(Intent::Clean);
                },
                "Clean"
            }
        }
    }
}
