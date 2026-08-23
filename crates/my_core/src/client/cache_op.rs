use crate::client;
use crate::client::fetches;
use crate::client::utility::cache::Cache;
use crate::client::utility::client_traits::ViewAndCache;
use crate::domain::request_response;
use crate::domain::use_cases;
use crate::domain::utility::resource_utils;
use crate::domain::utility::types::RowId;
use crate::domain::utility::uuid::User;
use crate::utility::traits;

pub trait DbBundle<Ch: Cache>: 'static {
    type CreateAccount: for<'a> use_cases::create_account::DatabaseRead<Db<'a> = Ch>;
    type CreateAccountForBranch: for<'a> use_cases::create_account_for_branch::DatabaseRead<Db<'a> = Ch>;
    type CreateJournalEntry: for<'a> use_cases::create_journal_entry::DatabaseRead<Db<'a> = Ch>;
    type GetAllAccountsForBranch: for<'a> use_cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Ch>;
    type CreateCompany: for<'a> use_cases::create_company::DatabaseRead<Db<'a> = Ch>;
    type CreateCompanyBranch: for<'a> use_cases::create_company_branch::DatabaseRead<Db<'a> = Ch>;
    type ListCompanyAndBranch: for<'a> use_cases::list_company_and_branch::DatabaseRead<Db<'a> = Ch>
        + 'static;
    type SignIn: for<'a> use_cases::sign_in::DatabaseRead<Db<'a> = Ch>;
    type SignUp: for<'a> use_cases::sign_up::DatabaseRead<Db<'a> = Ch>;
}

pub(crate) async fn new<Id: RowId, Ti: traits::Time, Dbb: DbBundle<Ch>, Ch: Cache>() -> Ch {
    let mut cache = Ch::new().await;
    let txns = cache.get_all_txn_input().await;

    for op in txns {
        op.operation.run_operation_check_apply::<Id, Ti, Ch, Dbb>(&mut cache).await;
    }

    cache
}

macro_rules! run_operation_check {
    ($path:ident, $name:ident, $db:ty, $i:expr, $state:expr) => {
        request_response::OperationsResult::$name(
            <client::use_cases::$path::ViewAndCacheType as ViewAndCache<Ch, $db>>::state_full_operation::<
                Id,
            >($i, $state)
            .await,
        )
    };
}

macro_rules! run_operation_check_apply {
    ($path:ident, $db:ty, $i:expr, $state:expr) => {
        let a= <client::use_cases::$path::ViewAndCacheType as ViewAndCache<Ch,$db>>::state_full_operation::<Id>($i, $state).await;
        let resources=<client::use_cases::$path::ViewAndCacheType as ViewAndCache<Ch,$db>>::extract_resource(&a);
        $state.write_resource_of_pending_txn(&resources).await;
    };
}

impl request_response::OperationsInput {
    pub(crate) async fn run_operation_check<
        Id: RowId,
        Ti: traits::Time,
        Ch: Cache,
        Dbb: DbBundle<Ch>,
    >(
        &self,
        state: &mut Ch,
    ) -> request_response::OperationsResult {
        match self {
            request_response::OperationsInput::SignUp(i) => {
                run_operation_check!(sign_up, SignUp, Dbb::SignUp, i, state)
            }
            request_response::OperationsInput::SignIn(i) => {
                run_operation_check!(sign_in, SignIn, Dbb::SignIn, i, state)
            }
            request_response::OperationsInput::CreateCompany(i) => {
                run_operation_check!(create_company, CreateCompany, Dbb::CreateCompany, i, state)
            }
            request_response::OperationsInput::CreateCompanyBranch(i) => {
                run_operation_check!(
                    create_company_branch,
                    CreateCompanyBranch,
                    Dbb::CreateCompanyBranch,
                    i,
                    state
                )
            }
            request_response::OperationsInput::ListCompanyAndBranch(i) => {
                run_operation_check!(
                    list_company_and_branch,
                    ListCompanyAndBranch,
                    Dbb::ListCompanyAndBranch,
                    i,
                    state
                )
            }
            request_response::OperationsInput::CreateAccount(i) => {
                run_operation_check!(create_account, CreateAccount, Dbb::CreateAccount, i, state)
            }
            request_response::OperationsInput::GetAllAccounts(_) => {
                unreachable!()
            }
            request_response::OperationsInput::GetAllAccountsForBranch(i) => {
                request_response::OperationsResult::GetAllAccountsForBranch(
                    <fetches::get_all_accounts_for_branch::ViewAndCacheType as ViewAndCache<
                        Ch,
                        Dbb::GetAllAccountsForBranch,
                    >>::state_full_operation::<Id>(i, state)
                    .await,
                )
            }
            request_response::OperationsInput::CreateAccountForBranch(i) => {
                run_operation_check!(
                    create_account_for_branch,
                    CreateAccountForBranch,
                    Dbb::CreateAccountForBranch,
                    i,
                    state
                )
            }
            request_response::OperationsInput::CreateJournalEntry(i) => {
                request_response::OperationsResult::CreateJournalEntry(
                    <client::use_cases::create_journal_entry::ViewAndCacheType<Ti> as ViewAndCache<
                        Ch,
                        Dbb::CreateJournalEntry,
                    >>::state_full_operation::<Id>(i, state)
                    .await,
                )
            }
        }
    }

    pub(crate) async fn run_operation_check_apply<
        Id: RowId,
        Ti: traits::Time,
        Ch: Cache,
        Dbb: DbBundle<Ch>,
    >(
        &self,
        state: &mut Ch,
    ) {
        match self {
            request_response::OperationsInput::SignUp(i) => {
                run_operation_check_apply!(sign_up, Dbb::SignUp, i, state);
            }
            request_response::OperationsInput::SignIn(i) => {
                run_operation_check_apply!(sign_in, Dbb::SignIn, i, state);
            }
            request_response::OperationsInput::CreateCompany(i) => {
                run_operation_check_apply!(create_company, Dbb::CreateCompany, i, state);
            }
            request_response::OperationsInput::CreateCompanyBranch(i) => {
                run_operation_check_apply!(
                    create_company_branch,
                    Dbb::CreateCompanyBranch,
                    i,
                    state
                );
            }
            request_response::OperationsInput::ListCompanyAndBranch(i) => {
                run_operation_check_apply!(
                    list_company_and_branch,
                    Dbb::ListCompanyAndBranch,
                    i,
                    state
                );
            }
            request_response::OperationsInput::CreateAccount(i) => {
                run_operation_check_apply!(create_account, Dbb::CreateAccount, i, state);
            }
            request_response::OperationsInput::GetAllAccounts(_) => {
                unreachable!()
            }
            request_response::OperationsInput::GetAllAccountsForBranch(_) => {
                unreachable!()
            }
            request_response::OperationsInput::CreateAccountForBranch(i) => {
                run_operation_check_apply!(
                    create_account_for_branch,
                    Dbb::CreateAccountForBranch,
                    i,
                    state
                );
            }
            request_response::OperationsInput::CreateJournalEntry(i) => {
                let result =
                    <client::use_cases::create_journal_entry::ViewAndCacheType<Ti> as ViewAndCache<
                        Ch,
                        Dbb::CreateJournalEntry,
                    >>::state_full_operation::<Id>(i, state)
                    .await;
                let resources =
                    <client::use_cases::create_journal_entry::ViewAndCacheType<Ti> as ViewAndCache<
                        Ch,
                        Dbb::CreateJournalEntry,
                    >>::extract_resource(&result);
                state.write_resource_of_pending_txn(&resources).await;
            }
        }
    }

    pub(crate) fn get_user_uuid<Ti: traits::Time, Ch: Cache, Dbb: DbBundle<Ch>>(
        &self,
    ) -> Option<&User> {
        match self {
            request_response::OperationsInput::SignUp(i) => {
                client::use_cases::sign_up::user_uuid(i)
            }
            request_response::OperationsInput::SignIn(i) => {
                client::use_cases::sign_in::user_uuid(i)
            }
            request_response::OperationsInput::CreateCompany(i) => {
                client::use_cases::create_company::user_uuid(i)
            }
            request_response::OperationsInput::CreateCompanyBranch(i) => {
                client::use_cases::create_company_branch::user_uuid(i)
            }
            request_response::OperationsInput::ListCompanyAndBranch(i) => {
                client::use_cases::list_company_and_branch::user_uuid(i)
            }
            request_response::OperationsInput::CreateAccount(i) => {
                client::use_cases::create_account::user_uuid(i)
            }
            request_response::OperationsInput::GetAllAccounts(i) => {
                client::fetches::get_all_accounts::user_uuid(i)
            }
            request_response::OperationsInput::GetAllAccountsForBranch(i) => {
                client::fetches::get_all_accounts_for_branch::user_uuid(i)
            }
            request_response::OperationsInput::CreateAccountForBranch(i) => {
                client::use_cases::create_account_for_branch::user_uuid(i)
            }
            request_response::OperationsInput::CreateJournalEntry(i) => {
                client::use_cases::create_journal_entry::user_uuid(i)
            }
        }
    }
}

macro_rules! extract_resource {
    ($path:ident, $db:ty, $i:expr) => {
        <client::use_cases::$path::ViewAndCacheType as ViewAndCache<Ch, $db>>::extract_resource($i)
    };
}

impl request_response::OperationsResult {
    pub(crate) fn extract_resource<Ti: traits::Time, Ch: Cache, Dbb: DbBundle<Ch>>(
        &self,
    ) -> Vec<resource_utils::ResourceInfo> {
        match self {
            request_response::OperationsResult::SignIn(i) => {
                extract_resource!(sign_in, Dbb::SignIn, i)
            }
            request_response::OperationsResult::SignUp(i) => {
                extract_resource!(sign_up, Dbb::SignUp, i)
            }
            request_response::OperationsResult::CreateCompany(i) => {
                extract_resource!(create_company, Dbb::CreateCompany, i)
            }
            request_response::OperationsResult::CreateCompanyBranch(i) => {
                extract_resource!(create_company_branch, Dbb::CreateCompanyBranch, i)
            }
            request_response::OperationsResult::ListCompanyAndBranch(i) => {
                extract_resource!(list_company_and_branch, Dbb::ListCompanyAndBranch, i)
            }
            request_response::OperationsResult::CreateAccount(i) => {
                extract_resource!(create_account, Dbb::CreateAccount, i)
            }
            request_response::OperationsResult::GetAllAccounts(i) => {
                fetches::get_all_accounts::extract_resource(i)
            }
            request_response::OperationsResult::GetAllAccountsForBranch(i) => {
                <fetches::get_all_accounts_for_branch::ViewAndCacheType as ViewAndCache<
                    Ch,
                    Dbb::GetAllAccountsForBranch,
                >>::extract_resource(i)
            }
            request_response::OperationsResult::CreateAccountForBranch(i) => {
                extract_resource!(create_account_for_branch, Dbb::CreateAccountForBranch, i)
            }
            request_response::OperationsResult::CreateJournalEntry(i) => {
                <client::use_cases::create_journal_entry::ViewAndCacheType<Ti> as ViewAndCache<
                    Ch,
                    Dbb::CreateJournalEntry,
                >>::extract_resource(i)
            }
        }
    }

    pub(crate) fn is_ok(&self) -> bool {
        match self {
            request_response::OperationsResult::SignUp(i) => i.is_ok(),
            request_response::OperationsResult::SignIn(i) => i.is_ok(),
            request_response::OperationsResult::CreateCompany(i) => i.is_ok(),
            request_response::OperationsResult::CreateCompanyBranch(i) => i.is_ok(),
            request_response::OperationsResult::ListCompanyAndBranch(i) => i.is_ok(),
            request_response::OperationsResult::CreateAccount(i) => i.is_ok(),
            request_response::OperationsResult::GetAllAccounts(i) => i.is_ok(),
            request_response::OperationsResult::CreateAccountForBranch(i) => i.is_ok(),
            request_response::OperationsResult::GetAllAccountsForBranch(i) => i.is_ok(),
            request_response::OperationsResult::CreateJournalEntry(i) => i.is_ok(),
        }
    }
}
