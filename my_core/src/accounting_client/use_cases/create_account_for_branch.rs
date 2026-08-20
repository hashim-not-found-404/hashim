use crate::accounting_client::client_domain::cache;
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
use crate::utility::traits::Sender;
use crate::utility::utils::ReadAndSet;
use std::sync::Arc;

type Type1 = cases::create_account_for_branch::Input;
type Type2 = cases::create_account_for_branch::Input;
type Type3 = cases::create_account_for_branch::MyResult;
type Type4 = cases::create_account_for_branch::MyResult;

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
        state: &mut Ch,
    ) -> Self::Type3 {
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
                        row_uuid: ok.new_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFlowTypeFieldAccount(
                            ok.belong_to_account.clone(),
                        ),
                    },
                    resource_utils::ResourceInfo {
                        row_uuid: ok.new_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFlowTypeFieldCompanyBranch(
                            ok.belong_to_company_branch.clone(),
                        ),
                    },
                    resource_utils::ResourceInfo {
                        row_uuid: ok.new_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFlowTypeFieldInflowType(
                            ok.inflow_type,
                        ),
                    },
                    resource_utils::ResourceInfo {
                        row_uuid: ok.new_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFlowTypeFieldOutflowType(
                            ok.outflow_type,
                        ),
                    },
                ]
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
        Id: types::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        As: ui_model::AllSignalTypes,
        Ch: cache::Cache + 'static,
        LongCache: for<'a> cases::create_account_for_branch::DatabaseRead<Db<'a> = Ch> + 'static,
        LongCacheForGetAllAccountsForBranch: for<'a> cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Ch> + 'static,
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
        model.page_create_account_for_branch.list_of_available_account.put(accounts);
    }
}

fn spawn_listener<
    Rn: traits::RandomNumber,
    Rt: traits::Runtime,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
    Ch: cache::Cache,
    LongCacheForGetAllAccountsForBranch: for<'a> cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static ui_model::Model<As>,
    cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let data = cases::get_all_accounts_for_branch::Input {
        user_uuid:           model.user_uuid.read().clone().unwrap().clone(),
        company_branch_uuid: model.selected_company_branch.read().unwrap().clone(),
    };
    let data = <fetches::get_all_accounts_for_branch::ViewAndCacheType as ViewAndCache<
        Ch,
        LongCacheForGetAllAccountsForBranch,
    >>::wrap_input(data);

    let listener_aborter = client_traits::spawn_listener::<Rn, Rt, Mpsc>(
        cache,
        <fetches::get_all_accounts_for_branch::ViewAndCacheType as ViewAndCache<
            Ch,
            LongCacheForGetAllAccountsForBranch,
        >>::subs(),
        data,
        move |data| {
            let data = <fetches::get_all_accounts_for_branch::ViewAndCacheType as ViewAndCache<
                Ch,
                LongCacheForGetAllAccountsForBranch,
            >>::unwrap_output(data);

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
    Id: types::RowId,
    Mpsc: traits::MultiProducerSingleConsumer,
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

    let Some(input) = build_input::<Id, As>(model) else {
        model.page_create_account_for_branch.is_loading.reset();
        return;
    };

    let data = <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::wrap_input(input);

    client_traits::handle_fall_back::<Rn, Rt, Mpsc, As>(
        cache,
        commander_local_state,
        &model.page_create_account_for_branch.show_dialog,
        process_manager::ProcessName::CreateAccountForBranch,
        data,
        |data| {
            let result = <ViewAndCacheType as ViewAndCache<Ch, LongCache>>::unwrap_output(data);
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

fn build_input<Id: types::RowId, As: ui_model::AllSignalTypes>(
    model: &ui_model::Model<As>,
) -> Option<Type1> {
    let account_uuid = model
        .page_create_account_for_branch
        .list_of_available_account
        .read()
        .first()
        .map(|acc| acc.row_uuid.clone())?;

    Some(cases::create_account_for_branch::Input {
        user_uuid:                model.user_uuid.read().clone().unwrap().clone(),
        new_uuid:                 Id::generate().clone(),
        belong_to_account:        account_uuid.clone(),
        belong_to_company_branch: model.selected_company_branch.read().unwrap().clone(),
        outflow_type:             model.page_create_account_for_branch.outflow_type.read(),
        inflow_type:              model.page_create_account_for_branch.inflow_type.read(),
    })
}
