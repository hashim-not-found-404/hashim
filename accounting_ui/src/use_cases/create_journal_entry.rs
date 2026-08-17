use crate::utility::components;
use crate::utility::tools;
use dioxus::prelude::*;
use my_core::accounting_client::client_domain::ui_model;
use my_core::accounting_client::client_domain::ui_model::HashimSignal;
use my_core::accounting_domain::utility::accounting_stuff;
use std::str::FromStr;

#[component]
pub(crate) fn CreateJournalEntry() -> Element {
    use_effect(move || {
        tools::send(ui_model::Message::CreateJournalEntry(ui_model::CreateJournalEntry::Subscribe));
    });

    use_drop(move || {
        tools::send(ui_model::Message::CreateJournalEntry(
            ui_model::CreateJournalEntry::UnSubscribe,
        ));
    });

    let model = &tools::MODEL;
    let local_state = &model.page_create_journal_entry;

    let consent_callback = move |consent: ui_model::UserConsent| {
        tools::send(ui_model::Message::CreateJournalEntry(ui_model::CreateJournalEntry::Consent(
            consent,
        )));
    };

    rsx! {
        div {
            // Dialog for offline consent
            components::Dialog {
                consent_callback,
                operation_name: "create journal entry",
                show_dialog: local_state.show_dialog.clone(),
            }

            // Shared Entry ID (optional)
            div {
                label { "Shared Entry ID (optional)" }
                input {
                    placeholder: "UUID",
                    value: local_state.shared_entry_id.read(),
                    oninput: move |event| {
                        tools::send(
                            ui_model::Message::CreateJournalEntry(
                                ui_model::CreateJournalEntry::SetSharedEntryId(event.value()),
                            ),
                        );
                    },
                }
            }

            // Display validation errors
            if local_state.error_container_is_empty.read() {
                label { "Container cannot be empty" }
            }
            if local_state.not_all_entry_inferred.read() {
                label { "Not all entries are fully inferred" }
            }
            if local_state.some_account_are_not_inferred.read() {
                label { "Some accounts have not been selected" }
            }

            // Double entries
            for (double_idx, double) in local_state.double_entries.read().iter().enumerate() {
                div {
                    // Double-level errors
                    if double.entry_is_empty {
                        label { "Double entry cannot be empty" }
                    }
                    if double.you_need_to_split_the_entry {
                        label { "This entry needs to be split" }
                    }
                    if let Some(ref dnc) = double.debit_not_equal_credit {
                        label {
                            {
                                format!(
                                    "Debit and credit totals do not match: debit={}, credit={}",
                                    dnc.total_debit,
                                    dnc.total_credit,
                                )
                            }
                        }
                    }

                    // Single entries for this double
                    for (single_idx, single) in double.singles.iter().enumerate() {
                        div {
                            // Single-level errors (display as needed)
                            if single.quantity_and_amount_are_zero {
                                label { "Quantity and amount are both zero" }
                            }
                            if single.duplicate_account_in_entry {
                                label { "Duplicate account in entry" }
                            }
                            if single.inventory_is_empty {
                                label { "Inventory is empty" }
                            }
                            if single.the_amount_should_be_positive {
                                label { "Amount must be positive" }
                            }
                            if single.the_quantity_should_be_positive {
                                label { "Quantity must be positive" }
                            }
                            if single.quantity_not_equal_amount {
                                label { "Quantity does not equal amount" }
                            }
                            if single.quantity_not_equal_zero {
                                label { "Quantity must be zero" }
                            }
                            if let Some(insufficient) = single.insufficient_quantity_in_inventory {
                                label {
                                    {format!("Insufficient quantity in inventory: available {}", insufficient)}
                                }
                            }
                            if let Some(mismatch) = single.amount_mismatch {
                                label { {format!("Amount mismatch: expected {}", mismatch)} }
                            }
                            if let Some(insufficient_amount) = single.insufficient_amount_in_inventory {
                                label {
                                    {format!("Insufficient amount in inventory: available {}", insufficient_amount)}
                                }
                            }

                            // Account selection with autocomplete
                            div {
                                input {
                                    placeholder: "Account Name",
                                    value: single.user_input_account_name.clone(),
                                    oninput: move |event| {
                                        let name = event.value();
                                        tools::send(
                                            ui_model::Message::CreateJournalEntry(ui_model::CreateJournalEntry::UpdateSingleEntry {
                                                double_index: double_idx,
                                                single_index: single_idx,
                                                value: ui_model::SingleEntryField::Account(name),
                                            }),
                                        );
                                    },
                                }
                                // Suggestions dropdown
                                for suggestion in local_state.filtered_list.read() {
                                    div {
                                        onclick: move |_| {
                                            tools::send(
                                                ui_model::Message::CreateJournalEntry(ui_model::CreateJournalEntry::SelectSuggestion {
                                                    double_index: double_idx,
                                                    single_index: single_idx,
                                                    account_uuid: suggestion.row_uuid.clone(),
                                                }),
                                            );
                                        },
                                        "{suggestion.account_name}"
                                    }
                                }
                            }

                            // Other fields
                            div {
                                label { "Is Debit" }
                                input {
                                    r#type: "checkbox",
                                    checked: single.user_input_is_debit.unwrap_or(false),
                                    onchange: move |event| {
                                        let checked = event.value().parse().unwrap_or(false);
                                        tools::send(
                                            ui_model::Message::CreateJournalEntry(ui_model::CreateJournalEntry::UpdateSingleEntry {
                                                double_index: double_idx,
                                                single_index: single_idx,
                                                value: ui_model::SingleEntryField::IsDebit(checked),
                                            }),
                                        );
                                    },
                                }
                            }
                            div {
                                label { "Is Inflow" }
                                input {
                                    r#type: "checkbox",
                                    checked: single.user_input_is_inflow.unwrap_or(false),
                                    onchange: move |event| {
                                        let checked = event.value().parse().unwrap_or(false);
                                        tools::send(
                                            ui_model::Message::CreateJournalEntry(ui_model::CreateJournalEntry::UpdateSingleEntry {
                                                double_index: double_idx,
                                                single_index: single_idx,
                                                value: ui_model::SingleEntryField::IsInflow(checked),
                                            }),
                                        );
                                    },
                                }
                            }
                            div {
                                label { "Inflow Type" }
                                select {
                                    value: single.user_input_inflow_type.as_ref().map(|t| t.as_str()).unwrap_or("Manual"),
                                    onchange: move |event| {
                                        let value = event.value();
                                        let inflow_type = accounting_stuff::InFlowType::from_str(&value)
                                            .unwrap_or_default();
                                        tools::send(
                                            ui_model::Message::CreateJournalEntry(ui_model::CreateJournalEntry::UpdateSingleEntry {
                                                double_index: double_idx,
                                                single_index: single_idx,
                                                value: ui_model::SingleEntryField::InflowType(inflow_type),
                                            }),
                                        );
                                    },
                                    option { value: "Manual", "Manual" }
                                    option { value: "QuantityEqualAmount", "Quantity Equal Amount" }
                                    option { value: "QuantityEqualZero", "Quantity Equal Zero" }
                                }
                            }
                            div {
                                label { "Outflow Type" }
                                select {
                                    value: single.user_input_outflow_type.as_ref().map(|t| t.as_str()).unwrap_or("Manual"),
                                    onchange: move |event| {
                                        let value = event.value();
                                        let outflow_type = accounting_stuff::OutFlowType::from_str(&value)
                                            .unwrap_or_default();
                                        tools::send(
                                            ui_model::Message::CreateJournalEntry(ui_model::CreateJournalEntry::UpdateSingleEntry {
                                                double_index: double_idx,
                                                single_index: single_idx,
                                                value: ui_model::SingleEntryField::OutflowType(outflow_type),
                                            }),
                                        );
                                    },
                                    option { value: "Manual", "Manual" }
                                    option { value: "QuantityEqualAmount", "Quantity Equal Amount" }
                                    option { value: "QuantityEqualZero", "Quantity Equal Zero" }
                                    option { value: "Wac", "WAC" }
                                    option { value: "Fifo", "FIFO" }
                                    option { value: "Lifo", "LIFO" }
                                    option { value: "Hifo", "HIFO" }
                                    option { value: "Lofo", "LOFO" }
                                }
                            }
                            div {
                                label { "Amount" }
                                input {
                                    r#type: "number",
                                    value: single.user_input_amount.map(|f| f.to_string()).unwrap_or_default(),
                                    oninput: move |event| {
                                        let val = event.value();
                                        let amount = val.parse().ok();
                                        tools::send(
                                            ui_model::Message::CreateJournalEntry(ui_model::CreateJournalEntry::UpdateSingleEntry {
                                                double_index: double_idx,
                                                single_index: single_idx,
                                                value: ui_model::SingleEntryField::Amount(amount.unwrap_or(0.0)),
                                            }),
                                        );
                                    },
                                }
                            }
                            div {
                                label { "Quantity" }
                                input {
                                    r#type: "number",
                                    value: single.user_input_quantity.map(|f| f.to_string()).unwrap_or_default(),
                                    oninput: move |event| {
                                        let val = event.value();
                                        let quantity = val.parse().ok();
                                        tools::send(
                                            ui_model::Message::CreateJournalEntry(ui_model::CreateJournalEntry::UpdateSingleEntry {
                                                double_index: double_idx,
                                                single_index: single_idx,
                                                value: ui_model::SingleEntryField::Quantity(quantity.unwrap_or(0.0)),
                                            }),
                                        );
                                    },
                                }
                            }

                            // Remove single entry button
                            button {
                                onclick: move |_| {
                                    tools::send(
                                        ui_model::Message::CreateJournalEntry(ui_model::CreateJournalEntry::RemoveSingleEntry {
                                            double_index: double_idx,
                                            single_index: single_idx,
                                        }),
                                    );
                                },
                                "Remove Single Entry"
                            }
                        }
                    }

                    // Add single entry button
                    button {
                        onclick: move |_| {
                            tools::send(
                                ui_model::Message::CreateJournalEntry(ui_model::CreateJournalEntry::AddSingleEntry {
                                    double_index: double_idx,
                                })
                            );
                        },
                        "Add Single Entry"
                    }
                }

                // Remove double entry button
                button {
                    onclick: move |_| {
                        tools::send(
                            ui_model::Message::CreateJournalEntry(ui_model::CreateJournalEntry::RemoveDoubleEntry {
                                double_index: double_idx,
                            })
                        );
                    },
                    "Remove Double Entry"
                }
            }

            // Add double entry button
            button {
                onclick: move |_| {
                    tools::send(
                        ui_model::Message::CreateJournalEntry(
                            ui_model::CreateJournalEntry::AddDoubleEntry,
                        ),
                    );
                },
                "Add Double Entry"
            }

            // Submit and Clean buttons
            button {
                disabled: local_state.is_loading.read(),
                onclick: move |_| {
                    tools::send(
                        ui_model::Message::CreateJournalEntry(ui_model::CreateJournalEntry::Submit),
                    );
                },
                "Submit"
            }
            button {
                onclick: move |_| {
                    tools::send(
                        ui_model::Message::CreateJournalEntry(ui_model::CreateJournalEntry::Clean),
                    );
                },
                "Clean"
            }
        }
    }
}
