use crate::accounting_client::client_domain::cache;
use crate::accounting_client::client_domain::cache_actor;
use crate::accounting_client::client_domain::client_traits;
use crate::accounting_client::client_domain::client_traits::ViewAndCache;
use crate::accounting_client::client_domain::commander;
use crate::accounting_client::client_domain::process_manager;
use crate::accounting_client::client_domain::ui_model;
use crate::accounting_client::client_domain::ui_model::HashimSignal;
use crate::accounting_client::fetches;
use crate::accounting_domain::cases;
use crate::accounting_domain::cases::create_journal_entry;
use crate::accounting_domain::request_response;
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types;
use crate::utility::tools;
use crate::utility::traits;
use crate::utility::traits::JoinHandle;
use crate::utility::traits::Receiver;
use crate::utility::traits::Sender;
use crate::utility::utils::ReadAndSet;
use std::marker::PhantomData;
use std::sync::Arc;

type Type1 = cases::create_journal_entry::Input;
type Type2 = cases::create_journal_entry::Input;
type Type3 = cases::create_journal_entry::MyResult;
type Type4 = cases::create_journal_entry::MyResult;

// -----------------------------------------------------------------------------
// Cache implementation that merges pending transactions
// -----------------------------------------------------------------------------

struct Cache<Ch, LongCache>
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_journal_entry::DatabaseRead<Db<'a> = Ch>,
{
    _ph: PhantomData<(Ch, LongCache)>,
}

impl<Ch, LongCache> cases::create_journal_entry::DatabaseRead for Cache<Ch, LongCache>
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_journal_entry::DatabaseRead<Db<'a> = Ch>,
{
    type Db<'a> = cache::State<Ch>;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::create_journal_entry::ReadInput,
    ) -> Result<cases::create_journal_entry::ReadOutput, traits::DynamicError> {
        let mut read_output = LongCache::read(&mut db.cache, read_input).await?;

        for (_, acf) in &db.state_of_pending_txn.access_control_for_company {
            if acf.user_ == read_input.user_uuid {
                read_output.user_roles.push(acf.role.clone());
            }
        }
        for (_, acfb) in &db.state_of_pending_txn.access_control_for_company_branch {
            if acfb.user_ == read_input.user_uuid {
                read_output.user_roles.push(acfb.role.clone());
            }
        }

        if db.state_of_pending_txn.entry.contains_key(&read_input.new_uuid) {
            read_output.is_new_uuid_used = true;
        }

        if let Some(shared_id) = &read_input.shared_entry_id {
            if db.state_of_pending_txn.shared_entry.contains_key(shared_id) {
                read_output.is_shared_entry_exist = true;
            }
        }

        for uuid in &read_input.new_entries_uuid {
            if db.state_of_pending_txn.entry.contains_key(uuid) {
                read_output.is_new_entries_uuid_used.insert(uuid.clone(), true);
            }
        }

        Ok(read_output)
    }
}

// -----------------------------------------------------------------------------
// ViewAndCache implementation
// -----------------------------------------------------------------------------

pub(crate) struct ViewAndCacheType<Ti: traits::Time>(Ti);

impl<Ch, LongCache, Ti> ViewAndCache<Ch, LongCache> for ViewAndCacheType<Ti>
where
    Ti: traits::Time,
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_journal_entry::DatabaseRead<Db<'a> = Ch>,
{
    type Type1 = Type1;
    type Type2 = Type2;
    type Type3 = Type3;
    type Type4 = Type4;

    fn wrap_input(data: Self::Type1) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::CreateJournalEntry(data)
    }

    fn user_uuid(data: &Self::Type2) -> Option<&types::UuidType> {
        Some(&data.user_uuid)
    }

    async fn state_full_operation<Id: types::RowId>(
        data: &Self::Type2,
        state: &mut cache::State<Ch>,
    ) -> Self::Type3 {
        data.state_full_check::<Cache<Ch, LongCache>, Ti>(state).await.unwrap()
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        match data {
            Ok(ok) => {
                let mut resources = Vec::new();

                // The journal entry itself (entry table)
                resources.push(resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.clone(),
                    resource: resource_utils::Resource::TableEntryFieldWriter(ok.user_uuid.clone()),
                });
                resources.push(resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.clone(),
                    resource: resource_utils::Resource::TableEntryFieldTime(ok.time),
                });
                if let Some(shared_id) = &ok.shared_entry_id {
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: ok.new_uuid.clone(),
                        resource: resource_utils::Resource::TableEntryFieldSharedEntryId(
                            shared_id.clone(),
                        ),
                    });
                }

                // Single entries
                for single in &ok.double_entry {
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: single.new_uuid.clone(),
                        resource: resource_utils::Resource::TableSingleEntryFieldDoubleEntry(
                            single.double_entry_number,
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: single.new_uuid.clone(),
                        resource: resource_utils::Resource::TableSingleEntryFieldEntry(
                            ok.new_uuid.clone(),
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: single.new_uuid.clone(),
                        resource: resource_utils::Resource::TableSingleEntryFieldAccount(
                            single.account.clone(),
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: single.new_uuid.clone(),
                        resource: resource_utils::Resource::TableSingleEntryFieldIsDebit(
                            single.is_debit,
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: single.new_uuid.clone(),
                        resource: resource_utils::Resource::TableSingleEntryFieldCostOutFlowType(
                            single.out_flow_type.clone(),
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: single.new_uuid.clone(),
                        resource: resource_utils::Resource::TableSingleEntryFieldQuantity(
                            single.quantity,
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: single.new_uuid.clone(),
                        resource: resource_utils::Resource::TableSingleEntryFieldAmount(
                            single.amount,
                        ),
                    });
                }

                for (account_uuid, inventory) in &ok.inventory {
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: account_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFieldInventory(
                            inventory.0.clone(),
                        ),
                    });
                }

                resources
            }
            Err(_) => Vec::new(),
        }
    }

    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self::Type4 {
        if let request_response::push_data::OperationsResult::CreateJournalEntry(result) = output {
            return result;
        }
        unreachable!()
    }

    fn apply_on_the_model<As: ui_model::AllSignalTypes>(
        output: &Self::Type4,
        model: &ui_model::Model<As>,
    ) {
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
                        // Map double-level errors
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

                        // Map single entry errors
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
                                ui_single.inventory_is_empty =
                                    domain_single_error.inventory_is_empty;
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
}

// -----------------------------------------------------------------------------
// UI Model update implementation
// -----------------------------------------------------------------------------

impl ui_model::CreateJournalEntry {
    pub(crate) async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: types::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
        Ti: traits::Time,
        As: ui_model::AllSignalTypes,
        Ch: cache::Cache + 'static,
        LongCache: for<'a> cases::create_journal_entry::DatabaseRead<Db<'a> = Ch> + 'static,
        LongCacheForGetAllAccountsForBranch: for<'a> cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Ch> + 'static,
    >(
        self,
        model: &'static ui_model::Model<As>,
        mut cache: client_traits::CacheActorStruct<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        let local_state = &model.page_create_journal_entry;

        match self {
            ui_model::CreateJournalEntry::Subscribe => {
                // 1. Fetch the list of available accounts (cache + server)
                let user_uuid = model.user_uuid.read().clone().unwrap();
                let company_branch_uuid = model.selected_company_branch.read().unwrap();

                let input = cases::get_all_accounts_for_branch::Input {
                    user_uuid:           user_uuid.clone(),
                    company_branch_uuid: company_branch_uuid.clone(),
                };

                let txn_number = Rn::generate();
                let mut receiver = cache
                    .send_to_cache_actor(
                        cache_actor::CachingStrategy::ReadCacheAndServer,
                        txn_number,
                        <fetches::get_all_accounts_for_branch::ViewAndCacheType as ViewAndCache<
                            Ch,
                            LongCacheForGetAllAccountsForBranch,
                        >>::wrap_input(input),
                    )
                    .await;

                // Wait for the first response
                if let cache_actor::Response::Data {
                    data,
                    ..
                } = receiver.recv().await.unwrap()
                {
                    let result =
                        <fetches::get_all_accounts_for_branch::ViewAndCacheType as ViewAndCache<
                            Ch,
                            LongCacheForGetAllAccountsForBranch,
                        >>::unwrap_output(data);
                    apply_fetch_result::<As>(&result, model);
                }

                // 2. Set up a subscription listener for live updates
                let listener_aborter =
                    handle_listener::<
                        Rn,
                        Rt,
                        Id,
                        Mpsc,
                        Rg,
                        As,
                        Ch,
                        LongCacheForGetAllAccountsForBranch,
                    >(model, cache.clone(), commander_local_state.clone());

                commander_local_state.aborter_to_accounts_listener.set(Box::new(listener_aborter));
            }
            ui_model::CreateJournalEntry::UnSubscribe => {
                commander_local_state.aborter_to_accounts_listener.abort();
            }
            ui_model::CreateJournalEntry::Submit => {
                handle_submit::<Rn, Rt, Id, Mpsc, Rg, Ti, As, Ch, LongCache>(
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
                if let Some(double) = entries.get_mut(double_index) {
                    if single_index < double.singles.len() {
                        double.singles.remove(single_index);
                        local_state.double_entries.set(entries);
                    }
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
                if let Some(double) = entries.get_mut(double_index) {
                    if let Some(single) = double.singles.get_mut(single_index) {
                        match value {
                            ui_model::SingleEntryField::Account(name) => {
                                // Filter the master list
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
            }
            ui_model::CreateJournalEntry::SetSharedEntryId(uuid_type) => {
                local_state.shared_entry_id.set(uuid_type);
            }
            ui_model::CreateJournalEntry::SelectSuggestion {
                double_index,
                single_index,
                account_uuid,
            } => {
                let mut entries = local_state.double_entries.read();
                if let Some(double) = entries.get_mut(double_index) {
                    if let Some(single) = double.singles.get_mut(single_index) {
                        // Find the account name from the master list
                        let master = local_state.list_of_available_account.lock().unwrap().clone();
                        if let Some(account) = master.iter().find(|a| a.row_uuid == account_uuid) {
                            single.user_input_account_name = account.account_name.clone();
                            single.inferred_account_id = Some(account_uuid);
                            local_state.filtered_list.set(Vec::new());
                        }
                        local_state.double_entries.set(entries);
                    }
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Helper functions for fetching and listener
// -----------------------------------------------------------------------------

/// Apply the fetch result to the model, converting domain accounts to UI accounts.
fn apply_fetch_result<As: ui_model::AllSignalTypes>(
    result: &cases::get_all_accounts_for_branch::MyResult,
    model: &ui_model::Model<As>,
) {
    if let Ok(ok) = result {
        let accounts: Vec<ui_model::Accounts> = ok
            .accounts
            .iter()
            .map(|a| {
                ui_model::Accounts {
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
        // Optionally reset filtered list? We'll leave it empty initially.
    }
}

/// Spawn a listener that re‑fetches whenever subscribed resources change.
fn handle_listener<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCacheForGetAllAccountsForBranch: for<'a> cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    mut cache: client_traits::CacheActorStruct<Mpsc>,
    _: Arc<commander::CommanderLocalState<Mpsc, As>>,
) -> impl FnOnce() {
    let component_id = Rn::generate() as u16;
    let mut cache1 = cache.clone();

    let mut handle = Rt::abortable_spawn_local(async move {
        // Subscribe to the relevant resources
        let mut receiver_to_poke = cache
            .send_subs_to_cache_actor(
                component_id,
                <fetches::get_all_accounts_for_branch::ViewAndCacheType as ViewAndCache<
                    Ch,
                    LongCacheForGetAllAccountsForBranch,
                >>::subs(),
            )
            .await;

        let user_uuid = model.user_uuid.read().clone().unwrap();
        let company_branch_uuid = model.selected_company_branch.read().unwrap();

        loop {
            if let Err(_) = receiver_to_poke.recv().await {
                break;
            }

            let input = cases::get_all_accounts_for_branch::Input {
                user_uuid:           user_uuid.clone(),
                company_branch_uuid: company_branch_uuid.clone(),
            };
            let txn_number = Rn::generate();
            let value = cache
                .send_to_cache_actor(
                    cache_actor::CachingStrategy::ReadCacheOnly,
                    txn_number,
                    <fetches::get_all_accounts_for_branch::ViewAndCacheType as ViewAndCache<
                        Ch,
                        LongCacheForGetAllAccountsForBranch,
                    >>::wrap_input(input),
                )
                .await
                .recv()
                .await
                .unwrap();

            let value = match value {
                cache_actor::Response::CloseTheChannel => break,
                cache_actor::Response::ServerCannotBeReached => break,
                cache_actor::Response::Data {
                    is_response_from_server: _,
                    data,
                } => {
                    <fetches::get_all_accounts_for_branch::ViewAndCacheType as ViewAndCache<
                        Ch,
                        LongCacheForGetAllAccountsForBranch,
                    >>::unwrap_output(data)
                }
            };

            apply_fetch_result::<As>(&value, model);

            if value.is_err() {
                break;
            }
        }

        // Clean up subscription when loop exits
        cache.send_unsubs_to_cache_actor(component_id).await;
    });

    // Return a closure that aborts the listener and unsubscribes
    move || {
        Rt::spawn_local(async move {
            handle.abort().await;
            cache1.send_unsubs_to_cache_actor(component_id).await;
        });
    }
}

// -----------------------------------------------------------------------------
// Clean and Submit (unchanged except for minor adjustments)
// -----------------------------------------------------------------------------

fn handle_clean<As: ui_model::AllSignalTypes>(model: &ui_model::Model<As>) {
    let local_state = &model.page_create_journal_entry;
    local_state.is_loading.reset();
    local_state.show_dialog.reset();
    local_state.double_entries.reset();
    local_state.some_account_are_not_inferred.reset();
    // Also clear the filtered list and master list? Master list is kept; filtered list reset.
    local_state.filtered_list.reset();
}

async fn handle_submit<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    Ti: traits::Time,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_journal_entry::DatabaseRead<Db<'a> = Ch>,
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

    let ui_entries = local_state.double_entries.read();
    for double in ui_entries.iter() {
        for single in double.singles.iter() {
            if single.inferred_account_id.is_none() {
                local_state.some_account_are_not_inferred.set(true);
                return;
            }
        }
    }

    // 2. Build the input (safe to unwrap now)
    let double_entries_input: Vec<cases::create_journal_entry::DoubleEntryInput> = ui_entries
        .into_iter()
        .map(|double| {
            let singles_input: Vec<cases::create_journal_entry::SingleEntryInput> = double
                .singles
                .into_iter()
                .map(|single| {
                    // safe because we validated above
                    let account = single.inferred_account_id.unwrap();
                    cases::create_journal_entry::SingleEntryInput {
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
            cases::create_journal_entry::DoubleEntryInput {
                single_entries: singles_input,
            }
        })
        .collect();

    let input = cases::create_journal_entry::Input {
        new_uuid:                 Id::generate(),
        belong_to_company_branch: model.selected_company_branch.read().unwrap(),
        user_uuid:                model.user_uuid.read().clone().unwrap(),
        shared_entry_id:          Id::parse(local_state.shared_entry_id.read()),
        double_entries:           double_entries_input,
    };

    let data = <ViewAndCacheType<Ti> as ViewAndCache<Ch, LongCache>>::wrap_input(input);
    let txn_number = Rn::generate();

    {
        let dialog: &'static As::Dialog = &local_state.show_dialog;
        let mut cache = cache;
        let data1 = data.clone();
        let mut cache1 = cache.clone();
        let commander_local_state1 = commander_local_state.clone();
        let mut handle = <Rt>::abortable_spawn_local(async move {
            let mut receiver_to_response = cache1
                .send_to_cache_actor(
                    cache_actor::CachingStrategy::WriteServerOnly,
                    txn_number,
                    data1,
                )
                .await;

            match receiver_to_response.recv().await.unwrap() {
                cache_actor::Response::CloseTheChannel => return,
                cache_actor::Response::ServerCannotBeReached => return,
                cache_actor::Response::Data {
                    is_response_from_server,
                    data,
                } => {
                    let result =
                        <ViewAndCacheType<Ti> as ViewAndCache<Ch, LongCache>>::unwrap_output(data);
                    let is_ok = result.is_ok();
                    <ViewAndCacheType<Ti> as ViewAndCache<Ch, LongCache>>::apply_on_the_model(
                        &result, model,
                    );

                    if is_ok {
                        handle_clean(model);
                    }

                    commander_local_state1
                        .sender_to_process_manager
                        .read()
                        .send(process_manager::MessageToProcessManager::FromProcess {
                            process_name: process_manager::ProcessName::CreateJournalEntry,
                            message:      process_manager::MessageFromProcess::Response {
                                is_response_from_server,
                                is_response_ok: is_ok,
                            },
                        })
                        .await
                        .unwrap();
                }
            }
        });

        let (sender_to_process, mut receiver_to_process) = <Mpsc>::channel();
        commander_local_state
            .sender_to_process_manager
            .read()
            .send(process_manager::MessageToProcessManager::FromProcess {
                process_name: process_manager::ProcessName::CreateJournalEntry,
                message:      process_manager::MessageFromProcess::Subscribe {
                    sender: sender_to_process,
                    dialog: &dialog,
                },
            })
            .await
            .unwrap();

        match receiver_to_process.recv().await.unwrap() {
            process_manager::MessageToProcess::FallBackToCache => {
                let mut receiver_to_response = cache
                    .send_to_cache_actor(
                        cache_actor::CachingStrategy::WriteCacheOnly,
                        txn_number,
                        data,
                    )
                    .await;

                match receiver_to_response.recv().await.unwrap() {
                    cache_actor::Response::CloseTheChannel => return,
                    cache_actor::Response::ServerCannotBeReached => return,
                    cache_actor::Response::Data {
                        is_response_from_server: _,
                        data,
                    } => {
                        let result =
                            <ViewAndCacheType<Ti> as ViewAndCache<Ch, LongCache>>::unwrap_output(
                                data,
                            );
                        let is_ok = result.is_ok();

                        <ViewAndCacheType<Ti> as ViewAndCache<Ch, LongCache>>::apply_on_the_model(
                            &result, model,
                        );

                        if is_ok {
                            handle_clean(model);
                        }
                    }
                }
            }
            process_manager::MessageToProcess::CancelOperation => {}
        }
        handle.abort().await;
    }

    local_state.is_loading.reset();
}
