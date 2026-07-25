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

        let mut roles = Vec::new();
        for (_, table) in &db.state_of_pending_txn.access_control_for_company {
            if read_input.user_uuid == table.user_ {
                roles.push(table.role.clone());
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
        match output {
            Ok(_) => {
                todo!()
            }
            Err(business_error) => {
                todo!()
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
        Ch: cache::Cache,
        LongCache: for<'a> cases::create_account_for_branch::DatabaseRead<Db<'a> = Ch>,
    >(
        self,
        model: &'static ui_model::Model<As>,
        cache: client_traits::CacheActorStruct<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    ) {
        match self {
            ui_model::CreateAccountForBranch::Show => {
                model.navigator.set(ui_model::Navigator::Home(ui_model::HomeNav {
                    show_menu: false,
                    page_to_present: ui_model::Menu::CreateAccountForBranch,
                }));
                fetches::get_all_accounts_for_branch::fetch::<Rn, Mpsc, As>(
                    cache,
                    commander_local_state,
                )
                .await;
            }
            ui_model::CreateAccountForBranch::Submit => todo!(),
            ui_model::CreateAccountForBranch::Consent(i) => {
                commander_local_state
                    .sender_to_process_manager
                    .read()
                    .send(process_manager::MessageToProcessManager::FromUser {
                        process_name: process_manager::ProcessName::CreateAccountForBranch,
                        consent: i,
                    })
                    .await
                    .unwrap();
            }
            ui_model::CreateAccountForBranch::Clean => todo!(),
            ui_model::CreateAccountForBranch::AccountName(i) => {
                let new_list = tools::select_strings(
                    model.page_create_account_for_branch.list_of_available_account.clone(),
                    i,
                );
                let mut first_element = String::new();

                if !new_list.is_empty() {
                    first_element = new_list[0].account_name.clone();
                }

                model.page_create_account_for_branch.filtered_list.set(new_list);
                model.page_create_account_for_branch.account_name.set(first_element);
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
