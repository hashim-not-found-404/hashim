use crate::client::Intent;
use accounting_engine::accounting_stuff::InFlowType;
use accounting_engine::accounting_stuff::OutFlowType;
use std::str::FromStr;
use use_case_prelude_ui::*;

#[component]
pub fn Component(
    sender: EventHandler<Intent>,
    show_dialog: Dialog,
    is_loading: bool,
    account_name: String,
    outflow_type: OutFlowType,
    inflow_type: InFlowType,
    list_of_account_name_and_uuid: Vec<(AccountUuid, String)>,
) -> Element {
    use_effect(move || {
        sender(Intent::Subscribe);
    });

    use_drop(move || {
        sender(Intent::UnSubscribe);
    });

    // you need to add field for select the company and field for branch
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
                operation_name: "create account for branch",
                show_dialog,
            }

            ListInput {
                disabled: is_loading,
                placeholder: "type account name",
                selected_item: account_name,
                on_input: move |a| sender(Intent::AccountName(a)),
                on_select: move |a| sender(Intent::SelectedAccount(a)),
                list: list_of_account_name_and_uuid.clone(),
                row_renderer: Callback::new(move |a: (AccountUuid, String)| {
                    rsx! {
                        button { disabled: is_loading, "{a.1}" }
                    }
                }),
            }

            select {
                disabled: is_loading,
                value: "{outflow_type.as_str()}",
                onchange: move |event| {
                    let t = OutFlowType::from_str(&event.value()).unwrap_or_default();
                    sender(Intent::OutflowType(t));
                },
                option { value: "Manual", "Manual" }
                option { value: "QuantityEqualAmount", "QuantityEqualAmount" }
                option { value: "QuantityEqualZero", "QuantityEqualZero" }
                option { value: "Wac", "Wac" }
                option { value: "Fifo", "Fifo" }
                option { value: "Lifo", "Lifo" }
                option { value: "Hifo", "Hifo" }
                option { value: "Lofo", "Lofo" }
            }

            select {
                disabled: is_loading,
                value: "{inflow_type.as_str()}",
                onchange: move |event| {
                    let t = InFlowType::from_str(&event.value()).unwrap_or_default();
                    sender(Intent::InflowType(t));
                },
                option { value: "Manual", "Manual" }
                option { value: "QuantityEqualAmount", "QuantityEqualAmount" }
                option { value: "QuantityEqualZero", "QuantityEqualZero" }
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
