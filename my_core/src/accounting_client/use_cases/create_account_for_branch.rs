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
use crate::accounting_domain::request_response;
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types;
use crate::accounting_domain::utility::types::MyErrorTrait;
use crate::utility::tools;
use crate::utility::traits;
use crate::utility::traits::JoinHandle;
use crate::utility::traits::Receiver;
use crate::utility::traits::Sender;
use crate::utility::utils::ReadAndSet;
use std::marker::PhantomData;
use std::sync::Arc;

type Type1 = cases::create_account_for_branch::Input;
type Type2 = cases::create_account_for_branch::Input;
type Type3 = cases::create_account_for_branch::MyResult;
type Type4 = cases::create_account_for_branch::MyResult;

struct Cache<Ch, LongCache>
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_account_for_branch::DatabaseRead<Db<'a> = Ch>,
{
    _ph: PhantomData<(Ch, LongCache)>,
}

impl<Ch, LongCache> cases::create_account_for_branch::DatabaseRead for Cache<Ch, LongCache>
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_account_for_branch::DatabaseRead<Db<'a> = Ch>,
{
    type Db<'a> = cache::State<Ch>;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::create_account_for_branch::ReadInput,
    ) -> Result<cases::create_account_for_branch::ReadOutput, traits::DynamicError> {
        let mut read_output = LongCache::read(&mut db.cache, read_input).await.unwrap();
        read_output.is_company_branch_exist = true;
        read_output.is_account_uuid_exist = true;
        read_output.is_new_uuid_used = false;

        for (row_uuid, table) in &db.state_of_pending_txn.account_flow_type {
            if read_input.belong_to_account == *row_uuid
                && read_input.belong_to_company_branch == table.company_branch
            {
                read_output.is_account_uuid_with_company_branch_used = true;
            }
        }

        for (_, table) in &db.state_of_pending_txn.access_control_for_company_branch {
            if read_input.user_uuid == table.user_ {
                read_output.user_roles.push(table.role.clone());
            }
        }

        Ok(read_output)
    }
}

pub(crate) struct ViewAndCacheType;

impl<Ch, LongCache> ViewAndCache<Ch, LongCache> for ViewAndCacheType
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_account_for_branch::DatabaseRead<Db<'a> = Ch>,
{
    type Type1 = Type1;
    type Type2 = Type2;
    type Type3 = Type3;
    type Type4 = Type4;

    fn wrap_input(data: Self::Type1) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::CreateAccountForBranch(data)
    }

    fn user_uuid(data: &Self::Type2) -> Option<&types::UuidType> {
        Some(&data.user_uuid)
    }

    async fn state_full_operation<Id: types::RowId>(
        data: &Self::Type2,
        state: &mut cache::State<Ch>,
    ) -> Self::Type3 {
        let errr = data.state_full_check::<Cache<Ch, LongCache>>(state).await.unwrap();

        if errr.is_there_error() {
            return Err(errr);
        }

        let ok = data.state_less_operation();

        return Ok(ok);
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        match data {
            Ok(ok) => {
                let mut resources = Vec::new();

                resources.push(resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.clone(),
                    resource: resource_utils::Resource::TableAccountFlowTypeFieldAccount(
                        ok.belong_to_account.clone(),
                    ),
                });
                resources.push(resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.clone(),
                    resource: resource_utils::Resource::TableAccountFlowTypeFieldCompanyBranch(
                        ok.belong_to_company_branch.clone(),
                    ),
                });
                resources.push(resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.clone(),
                    resource: resource_utils::Resource::TableAccountFlowTypeFieldInflowType(
                        ok.inflow_type.clone(),
                    ),
                });
                resources.push(resource_utils::ResourceInfo {
                    row_uuid: ok.new_uuid.clone(),
                    resource: resource_utils::Resource::TableAccountFlowTypeFieldOutflowType(
                        ok.outflow_type.clone(),
                    ),
                });

                resources
            }
            Err(_) => Vec::new(),
        }
    }

    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self::Type4 {
        if let request_response::push_data::OperationsResult::CreateAccountForBranch(result) =
            output
        {
            return result;
        }
        unreachable!("{:?}", output)
    }

    fn apply_on_the_model<As: ui_model::AllSignalTypes>(
        output: &Self::Type4,
        model: &ui_model::Model<As>,
    ) {
        let local_state = &model.page_create_account_for_branch;

        match output {
            Ok(_) => {
                local_state.is_loading.reset();
                local_state.show_dialog.reset();
                // Optionally clear other fields
                local_state.account_name.reset();
                local_state.outflow_type.reset();
                local_state.inflow_type.reset();
                // Keep the list of available accounts, but reset filtered list?
                local_state.filtered_list.reset();
            }
            Err(_) => {
                local_state.is_loading.reset();
            }
        }
    }
}

impl ui_model::CreateAccountForBranch {
    pub(crate) async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: types::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
        As: ui_model::AllSignalTypes,
        Ch: cache::Cache + 'static,
        LongCache: for<'a> cases::create_account_for_branch::DatabaseRead<Db<'a> = Ch> + 'static,
        LongCacheForGetAllAccountsForBranch: for<'a> cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Ch> + 'static,
    >(
        self,
        model: &'static ui_model::Model<As>,
        mut cache: client_traits::CacheActorStruct<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        match self {
            ui_model::CreateAccountForBranch::Subscribe => {
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
                        LongCache,
                        LongCacheForGetAllAccountsForBranch,
                    >(model, cache.clone(), commander_local_state.clone());

                // Store the aborter to cancel on unmount (requires adding field to CommanderLocalState)
                *commander_local_state.aborter_to_accounts_listener.lock().unwrap() =
                    Some(Box::new(listener_aborter));
            }
            ui_model::CreateAccountForBranch::UnSubscribe => {
                handle_unsubscribe(commander_local_state);
            }

            ui_model::CreateAccountForBranch::Submit => {
                handle_submit::<Rn, Rt, Id, Mpsc, Rg, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                )
                .await;
            }

            ui_model::CreateAccountForBranch::Clean => {
                handle_clean::<As>(model);
            }
            ui_model::CreateAccountForBranch::Consent(i) => {
                commander_local_state
                    .sender_to_process_manager
                    .read()
                    .send(process_manager::MessageToProcessManager::FromUser {
                        process_name: process_manager::ProcessName::CreateAccountForBranch,
                        consent:      i,
                    })
                    .await
                    .unwrap();
            }
            ui_model::CreateAccountForBranch::AccountName(i) => {
                let new_list = tools::select_strings(
                    model.page_create_account_for_branch.list_of_available_account.read(),
                    i.clone(),
                );

                model.page_create_account_for_branch.filtered_list.set(new_list);
                model.page_create_account_for_branch.account_name.set(i);
            }
            ui_model::CreateAccountForBranch::OutflowType(i) => {
                model.page_create_account_for_branch.outflow_type.set(i)
            }
            ui_model::CreateAccountForBranch::InflowType(i) => {
                model.page_create_account_for_branch.inflow_type.set(i)
            }
        }
    }
}

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
                    notes:                           a.notes.clone(),
                    unit_of_measurement_of_quantity: a.unit_of_measurement_of_quantity.clone(),
                }
            })
            .collect();
        model.page_create_account_for_branch.list_of_available_account.put(accounts);
        // Optionally reset the filtered list and account name?
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
    LongCache: for<'a> cases::create_account_for_branch::DatabaseRead<Db<'a> = Ch>,
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
            // Wait for a poke (data changed)
            let _ = receiver_to_poke.recv().await;

            // Re‑fetch from cache only (no server round trip)
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
                } => <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data),
            };

            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(&value, model);

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

// ---- Clean ----
fn handle_clean<As: ui_model::AllSignalTypes>(model: &ui_model::Model<As>) {
    let local_state = &model.page_create_account_for_branch;

    local_state.is_loading.reset();
    local_state.show_dialog.reset();
    local_state.account_name.reset();
    local_state.outflow_type.reset();
    local_state.inflow_type.reset();
    // Optionally clear filtered list as well
    local_state.filtered_list.reset();
    // Do NOT reset list_of_available_account – it's the master list from cache.
}

// ---- UnSubscribe ----
fn handle_unsubscribe<Mpsc: traits::MultiProducerSingleConsumer, As: ui_model::AllSignalTypes>(
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let mut guard = commander_local_state.aborter_to_accounts_listener.lock().unwrap();
    if let Some(f) = guard.take() {
        f();
    }
}

// ---- Submit ----
async fn handle_submit<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    Rg: traits::Regex,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCache: for<'a> cases::create_account_for_branch::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    if model.page_create_account_for_branch.is_loading.read() {
        return;
    }
    model.page_create_account_for_branch.is_loading.set(true);

    // 1. Gather inputs
    let user_uuid = model.user_uuid.read().clone().unwrap();
    let branch_uuid = model.selected_company_branch.read().unwrap();

    // Find the account UUID by matching the account name
    let account_uuid = model
        .page_create_account_for_branch
        .list_of_available_account
        .read()
        .first()
        .map(|acc| acc.row_uuid.clone());

    let account_uuid = match account_uuid {
        Some(uuid) => uuid,
        None => {
            // No matching account – set error and bail
            model.page_create_account_for_branch.is_loading.set(false);
            return;
        }
    };

    let new_uuid = Id::generate();

    let input = cases::create_account_for_branch::Input {
        user_uuid:                user_uuid.clone(),
        new_uuid:                 new_uuid.clone(),
        belong_to_account:        account_uuid.clone(),
        belong_to_company_branch: branch_uuid.clone(),
        outflow_type:             model.page_create_account_for_branch.outflow_type.read(),
        inflow_type:              model.page_create_account_for_branch.inflow_type.read(),
    };

    let data = <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(input);
    let txn_number = Rn::generate();

    {
        let dialog: &'static As::Dialog = &model.page_create_account_for_branch.show_dialog;
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
                        <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data);
                    let is_ok = result.is_ok();
                    <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(
                        &result, model,
                    );

                    if is_ok {
                        handle_clean(model);
                    }

                    commander_local_state1
                        .sender_to_process_manager
                        .read()
                        .send(process_manager::MessageToProcessManager::FromProcess {
                            process_name: process_manager::ProcessName::CreateAccountForBranch,
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
                process_name: process_manager::ProcessName::CreateAccountForBranch,
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
                            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data);
                        let is_ok = result.is_ok();

                        <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(
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

    // Reset the loading flag (already done in apply_on_the_model, but just in case)
    model.page_create_account_for_branch.is_loading.reset();
}
