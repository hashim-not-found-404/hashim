use crate::client::utility::cache::Cache;
use crate::client::utility::client_traits::OperationName;
use crate::domain::use_cases::get_all_accounts_for_branch::DatabaseRead;
use crate::domain::use_cases::get_all_accounts_for_branch::Input;
use crate::domain::use_cases::get_all_accounts_for_branch::MyResult;
use crate::domain::use_cases::get_all_accounts_for_branch::Ok;
use crate::domain::use_cases::get_all_accounts_for_branch::ReadInput;
use crate::domain::utility::uuid::User;
use crate::make_user_uuid;
use crate::make_wrap_unwrap;

make_wrap_unwrap!(get_all_accounts_for_branch, GetAllAccountsForBranch);
make_user_uuid!(get_all_accounts_for_branch);


type Type2 = Input;
type Type3 = MyResult;

pub(crate) async fn state_full_operation<
    Ch: Cache,
    LongCache: for<'a> DatabaseRead<Db<'a> = Ch>,
>(
    data: &Type2,
    state: &mut Ch,
) -> Type3 {
    let read_output = LongCache::read(
        state,
        &ReadInput {
            user_uuid: data.user_uuid.clone(),
            company_branch_uuid: data.company_branch_uuid.clone(),
        },
    )
    .await
    .unwrap();

    let ok = Ok {
        company_uuid: read_output.company_uuid,
        company_branch_uuid: data.company_branch_uuid.clone(),
        accounts: read_output.accounts,
        accounts_for_branch: read_output.accounts_for_branch,
    };
    Ok(ok)
}
