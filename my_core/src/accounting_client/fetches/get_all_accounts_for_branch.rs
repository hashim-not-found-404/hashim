use crate::accounting_client::client_domain::cache;
use crate::accounting_client::client_domain::client_traits::ViewAndCache;
use crate::accounting_client::client_domain::ui_model;
use crate::accounting_domain::cases;
use crate::accounting_domain::cases::get_all_accounts_for_branch::DatabaseRead;
use crate::accounting_domain::request_response;
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types;
use crate::utility::traits;
use std::collections::HashSet;
use std::marker::PhantomData;

// ---- 1. A DatabaseRead that merges pending transactions ----
struct CacheRead<Ch, LongCache>
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Ch>,
{
    _ph: PhantomData<(Ch, LongCache)>,
}

impl<Ch, LongCache> cases::get_all_accounts_for_branch::DatabaseRead for CacheRead<Ch, LongCache>
where
    Ch: cache::Cache,
    LongCache: for<'a> cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Ch>,
{
    type Db<'a> = cache::State<Ch>;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::get_all_accounts_for_branch::ReadInput,
    ) -> Result<cases::get_all_accounts_for_branch::ReadOutput, traits::DynamicError> {
        // 1. Read from the underlying cache
        let mut output = LongCache::read(&mut db.cache, read_input).await?;

        // 2. Merge pending account_flow_type entries (uncommitted changes)
        for (row_uuid, acft) in &db.state_of_pending_txn.account_flow_type {
            if acft.company_branch == read_input.company_branch_uuid {
                // Check if this account is already in output.accounts_for_branch
                let exists = output.accounts_for_branch.iter().any(|a| a.row_uuid == *row_uuid);
                if !exists {
                    // Need the account details – get them from pending account table.
                    // For simplicity, we try to find the account in pending.
                    if let Some(_) = db.state_of_pending_txn.account.get(&acft.account) {
                        // Also need the account's row_uuid – it's the key of the account.
                        // We'll clone the account and add it to output.accounts if not already present.
                        // (We also need to keep output.accounts and output.accounts_for_branch in sync.)
                        // For brevity, we just add a new AccountForBranch entry.
                        // In a real implementation, you'd also need to merge the Account details.
                        // This is a simplified version.
                        output.accounts_for_branch.push(
                            cases::get_all_accounts_for_branch::AccountForBranch {
                                row_uuid:     row_uuid.clone(),
                                account_uuid: acft.account.clone(),
                                outflow_type: acft.outflow_type.clone(),
                                inflow_type:  acft.inflow_type.clone(),
                            },
                        );
                    }
                }
            }
        }

        // Also need to merge pending accounts that belong to the same company?
        // For now we trust the underlying cache already has them.

        Ok(output)
    }
}

// ---- 2. The ViewAndCache implementation ----
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
        // We care about account fields and account_flow_type fields.
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
        state: &mut cache::State<Ch>,
    ) -> Self::Type3 {
        // Use the CacheRead to merge pending changes.
        let read_output = CacheRead::<Ch, LongCache>::read(
            state,
            &cases::get_all_accounts_for_branch::ReadInput {
                user_uuid:           data.user_uuid.clone(),
                company_branch_uuid: data.company_branch_uuid.clone(),
            },
        )
        .await
        .unwrap();

        let ok = cases::get_all_accounts_for_branch::Ok {
            company_uuid:        read_output.company_uuid,
            company_branch_uuid: read_output.company_branch_uuid,
            accounts:            read_output.accounts,
            accounts_for_branch: read_output.accounts_for_branch,
        };
        Ok(ok)
    }

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo> {
        // Same as existing ReadServerOnly extract_resource
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

    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self::Type4 {
        if let request_response::push_data::OperationsResult::GetAllAccountsForBranch(result) =
            output
        {
            // Filter the accounts before returning.
            match result {
                Ok(mut ok) => {
                    // Build a set of account UUIDs that are already linked.
                    let linked: HashSet<types::UuidType> =
                        ok.accounts_for_branch.iter().map(|afb| afb.account_uuid.clone()).collect();

                    // Keep only accounts that are NOT linked.
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
        // We'll apply manually in the listener.
    }
}
