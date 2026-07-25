use crate::accounting_client::client_domain::cache_actor;
use crate::accounting_client::client_domain::client_traits;
use crate::accounting_client::client_domain::client_traits::ReadServerOnly;
use crate::accounting_client::client_domain::commander;
use crate::accounting_client::client_domain::ui_model;
use crate::accounting_domain::cases;
use crate::accounting_domain::request_response;
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types;
use crate::utility::traits;
use crate::utility::utils::ReadAndSet;
use std::sync::Arc;

type Type1 = cases::get_all_accounts_for_branch::Input;
type Type2 = cases::get_all_accounts_for_branch::Input;
type Type3 = cases::get_all_accounts_for_branch::MyResult;

pub(crate) struct ViewAndCacheType;

impl ReadServerOnly for ViewAndCacheType {
    type Type1 = Type1;
    type Type2 = Type2;
    type Type3 = Type3;

    fn wrap_input(data: Self::Type1) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::GetAllAccountsForBranch(data)
    }

    fn user_uuid(data: &Self::Type2) -> Option<&types::UuidType> {
        Some(&data.user_uuid)
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        match data {
            Ok(ok) => {
                let mut resources = Vec::new();

                // For each account, add resources for every field we accessed.
                for account in &ok.accounts {
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: account.row_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFieldCompanyBelong(
                            ok.company_uuid.clone(),
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: account.row_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFieldIsDebit(
                            account.is_debit,
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: account.row_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFieldIsPermanentAccount(
                            account.is_permanent_account,
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: account.row_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFieldName(
                            account.account_name.clone(),
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: account.row_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFieldNotes(
                            account.notes.clone(),
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: account.row_uuid.clone(),
                        resource:
                            resource_utils::Resource::TableAccountFieldUnitOfMeasurementOfQuantity(
                                account.unit_of_measurement_of_quantity.clone(),
                            ),
                    });
                }

                // For each account_for_branch, add its resources.
                for acc_branch in &ok.accounts_for_branch {
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: acc_branch.row_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFlowTypeFieldAccount(
                            acc_branch.account_uuid.clone(),
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: acc_branch.row_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFlowTypeFieldInflowType(
                            acc_branch.inflow_type.clone(),
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: acc_branch.row_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFlowTypeFieldOutflowType(
                            acc_branch.outflow_type.clone(),
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: acc_branch.row_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFlowTypeFieldCompanyBranch(
                            ok.company_branch_uuid.clone(),
                        ),
                    });
                }

                resources
            }
            Err(_) => Vec::new(),
        }
    }
}

pub(crate) async fn fetch<
    Rn: traits::RandomNumber,
    Mpsc: traits::MultiProducerSingleConsumer,
    As: ui_model::AllSignalTypes,
>(
    mut cache: client_traits::CacheActorStruct<Mpsc>,
    commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
) {
    let input = cases::get_all_accounts_for_branch::Input {
        user_uuid:           commander_local_state.user_uuid.read().clone().unwrap(),
        company_branch_uuid: commander_local_state.selected_company_branch.read().unwrap(),
    };

    let txn_number = Rn::generate();

    cache
        .send_to_cache_actor(
            cache_actor::CachingStrategy::ReadServerOnly,
            txn_number,
            ViewAndCacheType::wrap_input(input),
        )
        .await;
}
