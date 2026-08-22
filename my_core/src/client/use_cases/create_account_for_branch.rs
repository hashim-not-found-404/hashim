use crate::client::fetches;
use crate::client::utility::cache;
use crate::client::utility::client_traits;
use crate::client::utility::client_traits::ViewAndCache;
use crate::client::utility::commander;
use crate::client::utility::process_manager;
use crate::client::utility::ui_model;
use crate::client::utility::ui_model::HashimSignal;
use crate::domain::use_cases;
use crate::domain::utility::resource_utils;
use crate::domain::utility::types::MyErrorTrait;
use crate::domain::utility::types::RowId;
use crate::domain::utility::uuid::User;
use crate::make_user_uuid;
use crate::make_wrap_unwrap;
use crate::utility::tools;
use crate::utility::traits;
use crate::utility::traits::Sender;
use crate::utility::utils::ReadAndSet;
use std::sync::Arc;

type Type1 = use_cases::create_account_for_branch::Input;
type Type2 = use_cases::create_account_for_branch::Input;
type Type3 = use_cases::create_account_for_branch::MyResult;
type Type4 = use_cases::create_account_for_branch::MyResult;

make_wrap_unwrap!(create_account_for_branch, CreateAccountForBranch);
make_user_uuid!(create_account_for_branch);

pub(crate) struct ViewAndCacheType;

impl<Ch, LongCache> ViewAndCache<Ch, LongCache> for ViewAndCacheType
where
    Ch: cache::Cache,
    LongCache: for<'a> use_cases::create_account_for_branch::DatabaseRead<Db<'a> = Ch>,
{
    type Type1 = Type1;
    type Type2 = Type2;
    type Type3 = Type3;
    type Type4 = Type4;

    async fn state_full_operation<Id: RowId>(data: &Self::Type2, state: &mut Ch) -> Self::Type3 {
        let errr = data.state_full_check::<LongCache>(state).await.unwrap();

        if errr.is_there_error() {
            return Err(errr);
        }

        Ok(data.state_less_operation())
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        match data {
            Ok(ok) => {
                vec![
                    resource_utils::ResourceInfo {
                        row_uuid: ok.new_uuid.0.clone(),
                        resource: resource_utils::Resource::TableAccountFlowTypeFieldAccount(
                            ok.belong_to_account.clone(),
                        ),
                    },
                    resource_utils::ResourceInfo {
                        row_uuid: ok.new_uuid.0.clone(),
                        resource: resource_utils::Resource::TableAccountFlowTypeFieldCompanyBranch(
                            ok.belong_to_company_branch.clone(),
                        ),
                    },
                    resource_utils::ResourceInfo {
                        row_uuid: ok.new_uuid.0.clone(),
                        resource: resource_utils::Resource::TableAccountFlowTypeFieldInflowType(
                            ok.inflow_type,
                        ),
                    },
                    resource_utils::ResourceInfo {
                        row_uuid: ok.new_uuid.0.clone(),
                        resource: resource_utils::Resource::TableAccountFlowTypeFieldOutflowType(
                            ok.outflow_type,
                        ),
                    },
                ]
            }
            Err(_) => Vec::new(),
        }
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
                local_state.account_name.reset();
                local_state.outflow_type.reset();
                local_state.inflow_type.reset();
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
        Id: RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        As: ui_model::AllSignalTypes,
        Ch: cache::Cache + 'static,
        LongCache: for<'a> use_cases::create_account_for_branch::DatabaseRead<Db<'a> = Ch> + 'static,
        LongCacheForGetAllAccountsForBranch: for<'a> use_cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Ch> + 'static,
    >(
        self,
        model: &'static ui_model::Model<As>,
        cache: client_traits::CacheActorStruct<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        match self {
            ui_model::CreateAccountForBranch::Subscribe => {
                spawn_listener::<Rn, Rt, Mpsc, As, Ch, LongCacheForGetAllAccountsForBranch>(
                    model,
                    cache.clone(),
                    commander_local_state,
                );
            }
            ui_model::CreateAccountForBranch::UnSubscribe => {
                commander_local_state.aborter_to_create_account_for_branch_listener.abort();
            }

            ui_model::CreateAccountForBranch::Submit => {
                handle_submit::<Rn, Rt, Id, Mpsc, As, Ch, LongCache>(
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
        model.page_create_account_for_branch.list_of_available_account.put(accounts);
    }
}

fn spawn_listener<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
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
        <fetches::get_all_accounts_for_branch::ViewAndCacheType as ViewAndCache<
            Ch,
            LongCacheForGetAllAccountsForBranch,
        >>::subs(),
        data,
        move |data| {
            let data = fetches::get_all_accounts_for_branch::unwrap_output(data);

            apply_fetch_result::<As>(&data, model);
        },
    );

    commander_local_state
        .aborter_to_create_account_for_branch_listener
        .set(Box::new(listener_aborter));
}

fn handle_clean<As: ui_model::AllSignalTypes>(model: &ui_model::Model<As>) {
    let local_state = &model.page_create_account_for_branch;

    local_state.is_loading.reset();
    local_state.show_dialog.reset();
    local_state.account_name.reset();
    local_state.outflow_type.reset();
    local_state.inflow_type.reset();
    local_state.filtered_list.reset();
}

async fn handle_submit<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Id: RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCache: for<'a> use_cases::create_account_for_branch::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    if model.page_create_account_for_branch.is_loading.read() {
        return;
    }
    model.page_create_account_for_branch.is_loading.set(true);

    let Some(input) = build_input::<Id, As>(model) else {
        model.page_create_account_for_branch.is_loading.reset();
        return;
    };

    let data = wrap_input(input);

    client_traits::handle_fall_back::<Rn, Rt, Mpsc, As>(
        cache,
        commander_local_state,
        &model.page_create_account_for_branch.show_dialog,
        process_manager::ProcessName::CreateAccountForBranch,
        data,
        |data| {
            let result = unwrap_output(data);
            <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::apply_on_the_model(&result, model);

            let is_ok = result.is_ok();
            if is_ok {
                handle_clean(model);
            }

            is_ok
        },
    )
    .await;

    model.page_create_account_for_branch.is_loading.reset();
}

fn build_input<Id: RowId, As: ui_model::AllSignalTypes>(
    model: &ui_model::Model<As>,
) -> Option<Type1> {
    let account_uuid = model
        .page_create_account_for_branch
        .list_of_available_account
        .read()
        .first()
        .map(|acc| acc.row_uuid.clone())?;

    Some(use_cases::create_account_for_branch::Input {
        user_uuid:                model.user_uuid.read().clone().unwrap().clone(),
        new_uuid:                 Id::generate().clone().into(),
        belong_to_account:        account_uuid.clone(),
        belong_to_company_branch: model.selected_company_branch.read().unwrap().clone(),
        outflow_type:             model.page_create_account_for_branch.outflow_type.read(),
        inflow_type:              model.page_create_account_for_branch.inflow_type.read(),
    })
}
