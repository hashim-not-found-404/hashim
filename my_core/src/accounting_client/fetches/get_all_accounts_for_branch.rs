use crate::accounting_client::client_domain::cache;
use crate::accounting_client::client_domain::client_traits::ViewAndCache;
use crate::accounting_client::client_domain::ui_model;
use crate::accounting_domain::cases;
use crate::accounting_domain::request_response;
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types;
use std::collections::HashSet;

pub struct ViewAndCacheType;

impl<Ch, LongCache> ViewAndCache<Ch, LongCache> for ViewAndCacheType
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Ch>,
{
    type Type1 = cases::get_all_accounts_for_branch::Input;
    type Type2 = cases::get_all_accounts_for_branch::Input;
    type Type3 = cases::get_all_accounts_for_branch::MyResult;
    type Type4 = cases::get_all_accounts_for_branch::MyResult;

    fn subs() -> &'static [resource_utils::Subscribe] {
        &[
            resource_utils::Subscribe::TableAccountFieldCompanyBelong,
            resource_utils::Subscribe::TableAccountFieldIsDebit,
            resource_utils::Subscribe::TableAccountFieldIsPermanentAccount,
            resource_utils::Subscribe::TableAccountFieldName,
            resource_utils::Subscribe::TableAccountFieldNotes,
            resource_utils::Subscribe::TableAccountFieldUnitOfMeasurementOfQuantity,
            resource_utils::Subscribe::TableAccountFlowTypeFieldAccount,
            resource_utils::Subscribe::TableAccountFlowTypeFieldCompanyBranch,
            resource_utils::Subscribe::TableAccountFlowTypeFieldInflowType,
            resource_utils::Subscribe::TableAccountFlowTypeFieldOutflowType,
        ]
    }

    fn wrap_input(data: Self::Type1) -> request_response::push_data::OperationsInput {
        request_response::push_data::OperationsInput::GetAllAccountsForBranch(data)
    }

    fn user_uuid(data: &Self::Type2) -> Option<&types::UuidType> {
        Some(&data.user_uuid)
    }

    async fn state_full_operation<Id: types::RowId>(
        data: &Self::Type2,
        state: &mut Ch,
    ) -> Self::Type3 {
        let read_output = LongCache::read(state, &cases::get_all_accounts_for_branch::ReadInput {
            user_uuid:           data.user_uuid.clone(),
            company_branch_uuid: data.company_branch_uuid.clone(),
        })
        .await
        .unwrap();

        let ok = cases::get_all_accounts_for_branch::Ok {
            company_uuid:        read_output.company_uuid,
            company_branch_uuid: data.company_branch_uuid.clone(),
            accounts:            read_output.accounts,
            accounts_for_branch: read_output.accounts_for_branch,
        };
        Ok(ok)
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        match data {
            Ok(ok) => {
                let mut resources = Vec::new();
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
                            acc_branch.inflow_type,
                        ),
                    });
                    resources.push(resource_utils::ResourceInfo {
                        row_uuid: acc_branch.row_uuid.clone(),
                        resource: resource_utils::Resource::TableAccountFlowTypeFieldOutflowType(
                            acc_branch.outflow_type,
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

    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self::Type4 {
        if let request_response::push_data::OperationsResult::GetAllAccountsForBranch(result) =
            output
        {
            match result {
                Ok(mut ok) => {
                    let linked: HashSet<types::UuidType> =
                        ok.accounts_for_branch.iter().map(|afb| afb.account_uuid.clone()).collect();

                    ok.accounts.retain(|acc| !linked.contains(&acc.row_uuid));

                    Ok(ok)
                }
                Err(e) => Err(e),
            }
        } else {
            unreachable!("{:?}", output)
        }
    }

    fn apply_on_the_model<As: ui_model::AllSignalTypes>(
        _output: &Self::Type4,
        _model: &ui_model::Model<As>,
    ) {
    }
}
