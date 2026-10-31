use crate::client::Intent;
use use_case_prelude_ui::*;

#[component]
pub fn Component(
    sender: EventHandler<Intent>,
    show_dialog: Dialog,
    is_loading: bool,
    company_name: String,
    company_name_error: Option<String>,
    resolved_company_uuid: Option<String>,
    name: String,
    name_error: Option<String>,
    is_it_contain_quantity: bool,
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
                operation_name: "create journal",
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
                    let bytes = a.0;
                    let bytes = bytes.to_string();
                    rsx! {
                        button { disabled: is_loading, "{a.1} [{bytes}]" }
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
                placeholder: "Journal Name",
                oninput: move |event| sender(Intent::Name(event.value())),
                value: name,
            }
            if let Some(err) = name_error {
                label { {err} }
            }

            div {
                label { "Does this journal track quantity?" }
                input {
                    disabled: is_loading,
                    r#type: "checkbox",
                    checked: is_it_contain_quantity,
                    onchange: move |event| {
                        sender(Intent::IsItContainQuantity(event.value().parse().unwrap_or_default()));
                    },
                }
            }

            button {
                disabled: is_loading,
                onclick: move |_| sender(Intent::Submit),
                "Submit"
            }
        }
    }
}
