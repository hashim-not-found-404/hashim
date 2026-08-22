use crate::accounting_client::client_domain::cache::Cache;
use crate::accounting_client::client_domain::client_traits;
use crate::accounting_client::client_domain::client_traits::CacheActorStruct;
use crate::accounting_client::client_domain::client_traits::ViewAndCache;
use crate::accounting_client::client_domain::client_traits::handle_fall_back;
use crate::accounting_client::client_domain::commander::CommanderLocalState;
use crate::accounting_client::client_domain::process_manager::MessageToProcessManager;
use crate::accounting_client::client_domain::process_manager::ProcessName;
use crate::accounting_client::client_domain::ui_model::Accounts;
use crate::accounting_client::client_domain::ui_model::AllSignalTypes;
use crate::accounting_client::client_domain::ui_model::CreateAccountForBranch;
use crate::accounting_client::client_domain::ui_model::HashimSignal;
use crate::accounting_client::client_domain::ui_model::Model;
use crate::accounting_client::fetches;
use crate::accounting_domain::cases;
use crate::accounting_domain::cases::create_account_for_branch::DatabaseRead;
use crate::accounting_domain::cases::create_account_for_branch::Input;
use crate::accounting_domain::cases::create_account_for_branch::MyResult;
use crate::accounting_domain::cases::create_account_for_branch::Ok;
use crate::accounting_domain::request_response::push_data::OperationsInput;
use crate::accounting_domain::request_response::push_data::OperationsResult;
use crate::accounting_domain::utility::types::MyErrorTrait;
use crate::accounting_domain::utility::types::RowId;
use crate::accounting_domain::utility::types::UuidType;
use crate::utility::tools::select_strings;
use crate::utility::traits::MultiProducerSingleConsumer;
use crate::utility::traits::RandomNumber;
use crate::utility::traits::Runtime;
use crate::utility::traits::Sender;
use crate::utility::utils::ReadAndSet;
use std::sync::Arc;

type Type1 = Input;
type Type2 = Input;
type Type3 = MyResult;
type Type4 = MyResult;
type StorableType = Ok;

pub(crate) struct ViewAndCacheType;

impl<Ch, LongCache> ViewAndCache<Ch, LongCache> for ViewAndCacheType
where
    Ch: Cache,
    LongCache: for<'a> DatabaseRead<Db<'a> = Ch>,
{
    type StorableType = StorableType;
    type Type1 = Type1;
    type Type2 = Type2;
    type Type3 = Type3;
    type Type4 = Type4;

    fn wrap_input(data: Self::Type1) -> OperationsInput {
        OperationsInput::CreateAccountForBranch(data)
    }

    fn user_uuid(data: &Self::Type2) -> Option<&UuidType> {
        Some(&data.user_uuid)
    }

    async fn state_full_operation<Id: RowId>(data: &Self::Type2, state: &mut Ch) -> Self::Type3 {
        let errr = data.state_full_check::<LongCache>(state).await.unwrap();

        if errr.is_there_error() {
            return Err(errr);
        }

        Ok(data.state_less_operation())
    }

    fn store_resource(data: &Self::Type3) -> Option<Self::StorableType> {
        match data {
            Ok(ok) => Some(ok.clone()),
            Err(_) => None,
        }
    }

    fn unwrap_output(output: OperationsResult) -> Self::Type4 {
        if let OperationsResult::CreateAccountForBranch(result) = output {
            return result;
        }
        unreachable!("{:?}", output)
    }

    fn apply_on_the_model<As: AllSignalTypes>(output: &Self::Type4, model: &Model<As>) {
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

impl CreateAccountForBranch {
    pub(crate) async fn update<
        Rn: RandomNumber,
        Rt: Runtime,
        Id: RowId,
        Mpsc: MultiProducerSingleConsumer,
        As: AllSignalTypes,
        Ch: Cache + 'static,
        LongCache: for<'a> DatabaseRead<Db<'a> = Ch> + 'static,
        LongCacheForGetAllAccountsForBranch: for<'a> cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Ch> + 'static,
    >(
        self,
        model: &'static Model<As>,
        cache: CacheActorStruct<Mpsc>,
        commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
    ) {
        match self {
            CreateAccountForBranch::Subscribe => {
                spawn_listener::<Rn, Rt, Mpsc, As, Ch, LongCacheForGetAllAccountsForBranch>(
                    model,
                    cache.clone(),
                    commander_local_state,
                );
            }
            CreateAccountForBranch::UnSubscribe => {
                commander_local_state.aborter_to_create_account_for_branch_listener.abort();
            }
            CreateAccountForBranch::Submit => {
                handle_submit::<Rn, Rt, Id, Mpsc, As, Ch, LongCache>(
                    model,
                    cache,
                    commander_local_state,
                )
                .await;
            }
            CreateAccountForBranch::Clean => {
                handle_clean::<As>(model);
            }
            CreateAccountForBranch::Consent(i) => {
                commander_local_state
                    .sender_to_process_manager
                    .read()
                    .send(MessageToProcessManager::FromUser {
                        process_name: ProcessName::CreateAccountForBranch,
                        consent:      i,
                    })
                    .await
                    .unwrap();
            }
            CreateAccountForBranch::AccountName(i) => {
                let new_list = select_strings(
                    model.page_create_account_for_branch.list_of_available_account.read(),
                    i.clone(),
                );

                model.page_create_account_for_branch.filtered_list.set(new_list);
                model.page_create_account_for_branch.account_name.set(i);
            }
            CreateAccountForBranch::OutflowType(i) => {
                model.page_create_account_for_branch.outflow_type.set(i)
            }
            CreateAccountForBranch::InflowType(i) => {
                model.page_create_account_for_branch.inflow_type.set(i)
            }
        }
    }
}

fn apply_fetch_result<As: AllSignalTypes>(
    result: &cases::get_all_accounts_for_branch::MyResult,
    model: &Model<As>,
) {
    if let Ok(ok) = result {
        let accounts: Vec<Accounts> = ok
            .accounts
            .iter()
            .map(|a| {
                Accounts {
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
    Rn: RandomNumber,
    Rt: Runtime,
    Mpsc: MultiProducerSingleConsumer,
    As: AllSignalTypes,
    Ch: Cache,
    LongCacheForGetAllAccountsForBranch: for<'a> cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static Model<As>,
    cache: CacheActorStruct<Mpsc>,
    commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
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

fn handle_clean<As: AllSignalTypes>(model: &Model<As>) {
    let local_state = &model.page_create_account_for_branch;

    local_state.is_loading.reset();
    local_state.show_dialog.reset();
    local_state.account_name.reset();
    local_state.outflow_type.reset();
    local_state.inflow_type.reset();
    local_state.filtered_list.reset();
}

async fn handle_submit<
    Rn: RandomNumber,
    Rt: Runtime,
    Id: RowId,
    Mpsc: MultiProducerSingleConsumer,
    As: AllSignalTypes,
    Ch: Cache,
    LongCache: for<'a> DatabaseRead<Db<'a> = Ch>,
>(
    model: &'static Model<As>,
    cache: CacheActorStruct<Mpsc>,
    commander_local_state: Arc<CommanderLocalState<Mpsc, As>>,
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

    handle_fall_back::<Rn, Rt, Mpsc, As>(
        cache,
        commander_local_state,
        &model.page_create_account_for_branch.show_dialog,
        ProcessName::CreateAccountForBranch,
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

fn build_input<Id: RowId, As: AllSignalTypes>(model: &Model<As>) -> Option<Type1> {
    let account_uuid = model
        .page_create_account_for_branch
        .list_of_available_account
        .read()
        .first()
        .map(|acc| acc.row_uuid.clone())?;

    Some(Input {
        user_uuid:                model.user_uuid.read().clone().unwrap().clone(),
        new_uuid:                 Id::generate().clone(),
        belong_to_account:        account_uuid.clone(),
        belong_to_company_branch: model.selected_company_branch.read().unwrap().clone(),
        outflow_type:             model.page_create_account_for_branch.outflow_type.read(),
        inflow_type:              model.page_create_account_for_branch.inflow_type.read(),
    })
}
