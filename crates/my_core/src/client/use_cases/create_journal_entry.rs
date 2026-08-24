use crate::client::fetches;
use crate::client::utility::cache::Cache;
use crate::client::utility::client_traits;
use crate::client::utility::commander;
use crate::client::utility::process_manager;
use crate::client::utility::ui_model;
use crate::client::utility::ui_model::HashimSignal;
use crate::domain::use_cases;
use crate::domain::use_cases::create_journal_entry;
use crate::domain::utility::types::RowId;
use crate::domain::utility::uuid::SharedEntry;
use crate::domain::utility::uuid::User;
use crate::make_user_uuid;
use crate::make_wrap_unwrap;
use crate::utility::tools;
use crate::utility::traits;
use crate::utility::traits::Sender;
use crate::utility::utils::ReadAndSet;
use std::sync::Arc;

type Type1 = use_cases::create_journal_entry::Input;
type Type2 = use_cases::create_journal_entry::Input;
type Type3 = use_cases::create_journal_entry::MyResult;
type Type4 = use_cases::create_journal_entry::MyResult;

make_wrap_unwrap!(create_journal_entry, CreateJournalEntry);
make_user_uuid!(create_journal_entry);

pub(crate) async fn state_full_operation<
    Ti: traits::Time,
    Ch: Cache,
    LongCache: for<'a> use_cases::create_journal_entry::DatabaseRead<Db<'a> = Ch>,
>(
    data: &Type2,
    state: &mut Ch,
) -> Type3 {
    data.state_full_check::<LongCache, Ti>(state).await.unwrap()
}

fn apply_on_the_model<As: ui_model::AllSignalTypes>(output: &Type4, model: &ui_model::Model<As>) {
    let local_state = &model.page_create_journal_entry;

    match output {
        Ok(_) => {}
        Err(business_error) => {
            local_state.error_container_is_empty.set(business_error.container_is_empty);
            local_state.not_all_entry_inferred.set(business_error.not_all_entry_inferred);

            let mut ui_double_entries = local_state.double_entries.read();

            if ui_double_entries.len() == business_error.double_entries.len() {
                for (ui_double, domain_double_error) in
                    ui_double_entries.iter_mut().zip(business_error.double_entries.iter())
                {
                    ui_double.entry_is_empty = domain_double_error.entry_is_empty;
                    ui_double.you_need_to_split_the_entry =
                        domain_double_error.you_need_to_split_the_entry;
                    ui_double.debit_not_equal_credit =
                        domain_double_error.debit_not_equal_credit.as_ref().map(|e| {
                            create_journal_entry::DebitNotEqualCreditError {
                                total_debit:  e.total_debit,
                                total_credit: e.total_credit,
                            }
                        });

                    if ui_double.singles.len() == domain_double_error.single_entries.len() {
                        for (ui_single, domain_single_error) in ui_double
                            .singles
                            .iter_mut()
                            .zip(domain_double_error.single_entries.iter())
                        {
                            ui_single.quantity_and_amount_are_zero =
                                domain_single_error.quantity_and_amount_are_zero;
                            ui_single.duplicate_account_in_entry =
                                domain_single_error.duplicate_account_in_entry;
                            ui_single.inventory_is_empty = domain_single_error.inventory_is_empty;
                            ui_single.the_amount_should_be_positive =
                                domain_single_error.the_amount_should_be_positive;
                            ui_single.the_quantity_should_be_positive =
                                domain_single_error.the_quantity_should_be_positive;
                            ui_single.quantity_not_equal_amount =
                                domain_single_error.quantity_not_equal_amount;
                            ui_single.quantity_not_equal_zero =
                                domain_single_error.quantity_not_equal_zero;
                            ui_single.insufficient_quantity_in_inventory =
                                domain_single_error.insufficient_quantity_in_inventory;
                            ui_single.amount_mismatch = domain_single_error.amount_mismatch;
                            ui_single.insufficient_amount_in_inventory =
                                domain_single_error.insufficient_amount_in_inventory;
                        }
                    }
                }
            }

            local_state.double_entries.set(ui_double_entries);
        }
    }
}

impl ui_model::CreateJournalEntry {
    pub(crate) async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Ti: traits::Time,
        As: ui_model::AllSignalTypes,
        Ch: Cache + 'static,
        LongCache: for<'a> use_cases::create_journal_entry::DatabaseRead<Db<'a> = Ch> + 'static,
        LongCacheForGetAllAccountsForBranch: for<'a> use_cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Ch> + 'static,
    >(
        self,
        model: &'static ui_model::Model<As>,
        cache: client_traits::CacheActorStruct<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        let local_state = &model.page_create_journal_entry;

        match self {
            ui_model::CreateJournalEntry::Subscribe => {
                spawn_listener::<Rn, Rt, Mpsc, As, Ch, LongCacheForGetAllAccountsForBranch>(
                    model,
                    cache.clone(),
                    commander_local_state,
                );
            }
            ui_model::CreateJournalEntry::UnSubscribe => {
                commander_local_state.aborter_to_create_journal_entry_listener.abort();
            }
            ui_model::CreateJournalEntry::Submit => {
                handle_submit::<Rn, Rt, Id, Mpsc, Ti, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                )
                .await;
            }
            ui_model::CreateJournalEntry::Consent(i) => {
                commander_local_state
                    .sender_to_process_manager
                    .read()
                    .send(process_manager::MessageToProcessManager::FromUser {
                        process_name: process_manager::ProcessName::CreateJournalEntry,
                        consent:      i,
                    })
                    .await
                    .unwrap();
            }
            ui_model::CreateJournalEntry::Clean => {
                handle_clean::<As>(model);
            }
            ui_model::CreateJournalEntry::AddSingleEntry {
                double_index,
            } => {
                let mut entries = local_state.double_entries.read();
                if let Some(double) = entries.get_mut(double_index) {
                    double.singles.push(ui_model::SingleEntry::default());
                    local_state.double_entries.set(entries);
                }
            }
            ui_model::CreateJournalEntry::RemoveSingleEntry {
                double_index,
                single_index,
            } => {
                let mut entries = local_state.double_entries.read();
                if let Some(double) = entries.get_mut(double_index)
                    && single_index < double.singles.len()
                {
                    double.singles.remove(single_index);
                    local_state.double_entries.set(entries);
                }
            }
            ui_model::CreateJournalEntry::AddDoubleEntry => {
                let mut entries = local_state.double_entries.read();
                entries.push(ui_model::DoubleEntry::default());
                local_state.double_entries.set(entries);
            }
            ui_model::CreateJournalEntry::RemoveDoubleEntry {
                double_index,
            } => {
                let mut entries = local_state.double_entries.read();
                if double_index < entries.len() {
                    entries.remove(double_index);
                    local_state.double_entries.set(entries);
                }
            }
            ui_model::CreateJournalEntry::UpdateSingleEntry {
                double_index,
                single_index,
                value,
            } => {
                let mut entries = local_state.double_entries.read();
                if let Some(double) = entries.get_mut(double_index)
                    && let Some(single) = double.singles.get_mut(single_index)
                {
                    match value {
                        ui_model::SingleEntryField::Account(name) => {
                            let master =
                                local_state.list_of_available_account.lock().unwrap().clone();
                            let filtered = tools::select_strings(master, name.clone());
                            local_state.filtered_list.set(filtered);
                            single.user_input_account_name = name;
                            single.inferred_account_id = None;
                        }
                        ui_model::SingleEntryField::IsDebit(b) => {
                            single.user_input_is_debit = Some(b);
                        }
                        ui_model::SingleEntryField::IsInflow(b) => {
                            single.user_input_is_inflow = Some(b);
                        }
                        ui_model::SingleEntryField::InflowType(t) => {
                            single.user_input_inflow_type = Some(t);
                        }
                        ui_model::SingleEntryField::OutflowType(t) => {
                            single.user_input_outflow_type = Some(t);
                        }
                        ui_model::SingleEntryField::Amount(f) => {
                            single.user_input_amount = Some(f);
                        }
                        ui_model::SingleEntryField::Quantity(f) => {
                            single.user_input_quantity = Some(f);
                        }
                    }
                    local_state.double_entries.set(entries);
                }
            }
            ui_model::CreateJournalEntry::SetSharedEntryId(uuid_type) => {
                local_state.shared_entry_id.set(uuid_type);
            }
            ui_model::CreateJournalEntry::SelectSuggestion {
                double_index: _,
                single_index: _,
                account_uuid: _,
            } => {
                // let mut entries = local_state.double_entries.read();
                // if let Some(double) = entries.get_mut(double_index)
                //     && let Some(single) = double.singles.get_mut(single_index)
                // {
                //     let master = local_state.list_of_available_account.lock().unwrap().clone();
                //     if let Some(account) = master.iter().find(|a| a.row_uuid == account_uuid) {
                //         single.user_input_account_name = account.account_name.clone();
                //         single.inferred_account_id = Some(account_uuid);
                //         local_state.filtered_list.set(Vec::new());
                //     }
                //     local_state.double_entries.set(entries);
                // }
            }
        }
    }
}

fn apply_fetch_result<As: ui_model::AllSignalTypes>(
    result: &use_cases::get_all_accounts_for_branch::MyResult,
    model: &ui_model::Model<As>,
) {
    if let Ok(ok) = result {
        let accounts: Vec<ui_model::Account> = ok
            .accounts
            .iter()
            .map(|a| {
                ui_model::Account {
                    row_uuid:                        a.row_uuid.clone(),
                    is_debit:                        a.is_debit,
                    is_permanent_account:            a.is_permanent_account,
                    account_name:                    a.account_name.clone(),
                    notes:                           a.notes.clone().unwrap_or_default(),
                    unit_of_measurement_of_quantity: a.unit_of_measurement_of_quantity.clone(),
                }
            })
            .collect();
        *model.page_create_journal_entry.list_of_available_account.lock().unwrap() = accounts;
    }
}

fn spawn_listener<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
    Ch: Cache,
    LongCacheForGetAllAccountsForBranch: for<'a> use_cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let data = use_cases::get_all_accounts_for_branch::Input {
        user_uuid:           model.user_uuid.read().clone().unwrap().clone(),
        company_branch_uuid: model.selected_company_branch.read().unwrap().clone(),
    };
    let data = fetches::get_all_accounts_for_branch::wrap_input(data);

    let listener_aborter = client_traits::spawn_listener::<Rn, Rt, Mpsc>(
        cache,
        fetches::get_all_accounts_for_branch::SUBSCRIBES_TO_LISTEN,
        data,
        move |data| {
            let data = fetches::get_all_accounts_for_branch::unwrap_output(data);

            apply_fetch_result::<As>(&data, model);
        },
    );

    commander_local_state.aborter_to_create_journal_entry_listener.set(Box::new(listener_aborter));
}

fn handle_clean<As: ui_model::AllSignalTypes>(model: &ui_model::Model<As>) {
    let local_state = &model.page_create_journal_entry;
    local_state.is_loading.reset();
    local_state.show_dialog.reset();
    local_state.double_entries.reset();
    local_state.some_account_are_not_inferred.reset();
    local_state.filtered_list.reset();
}

async fn handle_submit<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Ti: traits::Time,
    As: ui_model::AllSignalTypes,
    Ch: Cache,
    LongCache: for<'a> use_cases::create_journal_entry::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let local_state = &model.page_create_journal_entry;

    if local_state.is_loading.read() {
        return;
    }
    local_state.is_loading.set(true);

    let Some(input) = build_input::<Id, As>(model) else {
        model.page_create_journal_entry.is_loading.reset();
        return;
    };

    let data = wrap_input(input);

    client_traits::handle_fall_back::<Rn, Rt, Mpsc, As>(
        cache,
        commander_local_state,
        &model.page_create_journal_entry.show_dialog,
        process_manager::ProcessName::CreateJournalEntry,
        data,
        move |data| {
            let result = unwrap_output(data);
            apply_on_the_model(&result, model);

            let is_ok = result.is_ok();
            if is_ok {
                handle_clean(model);
            }

            is_ok
        },
    )
    .await;

    local_state.is_loading.reset();
}

fn build_input<Id: RowId, As: ui_model::AllSignalTypes>(
    model: &ui_model::Model<As>,
) -> Option<Type1> {
    let local_state = &model.page_create_journal_entry;
    let ui_entries = local_state.double_entries.read();
    for double in &ui_entries {
        for single in &double.singles {
            if single.inferred_account_id.is_none() {
                local_state.some_account_are_not_inferred.set(true);
                return None;
            }
        }
    }

    let double_entries_input: Vec<use_cases::create_journal_entry::DoubleEntryInput> = ui_entries
        .into_iter()
        .map(|double| {
            let singles_input: Vec<use_cases::create_journal_entry::SingleEntryInput> = double
                .singles
                .into_iter()
                .map(|single| {
                    let account = single.inferred_account_id.unwrap();
                    use_cases::create_journal_entry::SingleEntryInput {
                        new_uuid: Id::generate(),
                        account,
                        is_debit: single.user_input_is_debit,
                        is_inflow: single.user_input_is_inflow,
                        inflow_type: single.user_input_inflow_type,
                        outflow_type: single.user_input_outflow_type,
                        amount: single.user_input_amount,
                        quantity: single.user_input_quantity,
                    }
                })
                .collect();
            use_cases::create_journal_entry::DoubleEntryInput {
                single_entries: singles_input,
            }
        })
        .collect();

    Some(use_cases::create_journal_entry::Input {
        new_uuid:                 Id::generate(),
        belong_to_company_branch: model.selected_company_branch.read().unwrap(),
        user_uuid:                model.user_uuid.read().clone().unwrap(),
        shared_entry_id:          Id::parse(local_state.shared_entry_id.read())
            .map(|v| SharedEntry(v)),
        double_entries:           double_entries_input,
    })
}
