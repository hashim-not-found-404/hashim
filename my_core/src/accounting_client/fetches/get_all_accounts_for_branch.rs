use crate::accounting_client::client_domain::cache::Cache;
use crate::accounting_client::client_domain::client_traits::Subscribe;
use crate::accounting_client::client_domain::client_traits::ViewAndCache;
use crate::accounting_client::client_domain::ui_model::AllSignalTypes;
use crate::accounting_client::client_domain::ui_model::Model;
use crate::accounting_domain::cases::get_all_accounts_for_branch::DatabaseRead;
use crate::accounting_domain::cases::get_all_accounts_for_branch::Input;
use crate::accounting_domain::cases::get_all_accounts_for_branch::MyResult;
use crate::accounting_domain::cases::get_all_accounts_for_branch::Ok;
use crate::accounting_domain::cases::get_all_accounts_for_branch::ReadInput;
use crate::accounting_domain::request_response::push_data::OperationsInput;
use crate::accounting_domain::request_response::push_data::OperationsResult;
use crate::accounting_domain::utility::types::RowId;
use crate::accounting_domain::utility::types::UuidType;
use std::collections::HashSet;

pub struct ViewAndCacheType;

impl<Ch, LongCache> ViewAndCache<Ch, LongCache> for ViewAndCacheType
where
    Ch: Cache,
    LongCache: for<'a> DatabaseRead<Db<'a> = Ch>,
{
    type StorableType = Ok;
    type Type1 = Input;
    type Type2 = Input;
    type Type3 = MyResult;
    type Type4 = MyResult;

    fn subs() -> &'static [Subscribe] {
        &[
            Subscribe::TableAccountFieldCompanyBelong,
            Subscribe::TableAccountFieldIsDebit,
            Subscribe::TableAccountFieldIsPermanentAccount,
            Subscribe::TableAccountFieldName,
            Subscribe::TableAccountFieldNotes,
            Subscribe::TableAccountFieldUnitOfMeasurementOfQuantity,
            Subscribe::TableAccountFlowTypeFieldAccount,
            Subscribe::TableAccountFlowTypeFieldCompanyBranch,
            Subscribe::TableAccountFlowTypeFieldInflowType,
            Subscribe::TableAccountFlowTypeFieldOutflowType,
        ]
    }

    fn wrap_input(data: Self::Type1) -> OperationsInput {
        OperationsInput::GetAllAccountsForBranch(data)
    }

    fn user_uuid(data: &Self::Type2) -> Option<&UuidType> {
        Some(&data.user_uuid)
    }

    async fn state_full_operation<Id: RowId>(data: &Self::Type2, state: &mut Ch) -> Self::Type3 {
        let read_output = LongCache::read(state, &ReadInput {
            user_uuid:           data.user_uuid.clone(),
            company_branch_uuid: data.company_branch_uuid.clone(),
        })
        .await
        .unwrap();

        let ok = Ok {
            company_uuid:        read_output.company_uuid,
            company_branch_uuid: data.company_branch_uuid.clone(),
            accounts:            read_output.accounts,
            accounts_for_branch: read_output.accounts_for_branch,
        };
        Ok(ok)
    }

    fn extract_resource(data: &Self::Type3) -> Option<Self::StorableType> {
        match data {
            Ok(ok) => Some(ok.clone()),
            Err(_) => None,
        }
    }

    fn unwrap_output(output: OperationsResult) -> Self::Type4 {
        if let OperationsResult::GetAllAccountsForBranch(result) = output {
            match result {
                Ok(mut ok) => {
                    let linked: HashSet<UuidType> =
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

    fn apply_on_the_model<As: AllSignalTypes>(_output: &Self::Type4, _model: &Model<As>) {}
}
