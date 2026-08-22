use crate::accounting_client::client_domain::cache::Cache;
use crate::accounting_client::client_domain::client_traits::ReadServerOnly;
use crate::accounting_client::client_domain::client_traits::ViewAndCache;
use crate::accounting_client::fetches;
use crate::accounting_client::use_cases;
use crate::accounting_domain::cases;
use crate::accounting_domain::request_response::push_data::OperationsInput;
use crate::accounting_domain::request_response::push_data::OperationsResult;
use crate::accounting_domain::utility::types::DatabaseWrite;
use crate::accounting_domain::utility::types::RowId;
use crate::accounting_domain::utility::types::UuidType;
use crate::utility::traits::Time;

pub trait DbBundle<Ch: Cache>: 'static {
    type CreateAccount: for<'a> cases::create_account::DatabaseRead<Db<'a> = Ch>;
    type WriteCreateAccount: for<'a> DatabaseWrite<Db<'a> = Ch, Input = cases::create_account::Ok>;
    type CreateAccountForBranch: for<'a> cases::create_account_for_branch::DatabaseRead<Db<'a> = Ch>;
    type WriteCreateAccountForBranch: for<'a> DatabaseWrite<Db<'a> = Ch, Input = cases::create_account_for_branch::Ok>;
    type CreateJournalEntry: for<'a> cases::create_journal_entry::DatabaseRead<Db<'a> = Ch>;
    type WriteCreateJournalEntry: for<'a> DatabaseWrite<Db<'a> = Ch, Input = cases::create_journal_entry::Ok>;
    type GetAllAccountsForBranch: for<'a> cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Ch>;
    type WriteGetAllAccountsForBranch: for<'a> DatabaseWrite<Db<'a> = Ch, Input = cases::get_all_accounts_for_branch::Ok>;
    type CreateCompany: for<'a> cases::create_company::DatabaseRead<Db<'a> = Ch>;
    type WriteCreateCompany: for<'a> DatabaseWrite<Db<'a> = Ch, Input = cases::create_company::Ok>;
    type CreateCompanyBranch: for<'a> cases::create_company_branch::DatabaseRead<Db<'a> = Ch>;
    type WriteCreateCompanyBranch: for<'a> DatabaseWrite<Db<'a> = Ch, Input = cases::create_company_branch::Ok>;
    type ListCompanyAndBranch: for<'a> cases::list_company_and_branch::DatabaseRead<Db<'a> = Ch>
        + 'static;
    type WriteListCompanyAndBranch: for<'a> DatabaseWrite<Db<'a> = Ch, Input = cases::list_company_and_branch::Ok>
        + 'static;
    type SignIn: for<'a> cases::sign_in::DatabaseRead<Db<'a> = Ch>;
    type WriteSignIn: for<'a> DatabaseWrite<Db<'a> = Ch, Input = cases::sign_in::Ok>;
    type SignUp: for<'a> cases::sign_up::DatabaseRead<Db<'a> = Ch>;
    type WriteSignUp: for<'a> DatabaseWrite<Db<'a> = Ch, Input = cases::sign_up::Ok>;
}

pub(crate) async fn new<Id: RowId, Ti: Time, Dbb: DbBundle<Ch>, Ch: Cache>() -> Ch {
    let mut cache = Ch::new().await;
    let txns = cache.get_all_txn_input().await;

    for op in txns {
        op.operation.run_operation_check_apply::<Id, Ti, Ch, Dbb>(&mut cache).await;
    }

    cache
}

macro_rules! run_operation_check {
    ($path:ident, $name:ident, $db:ty, $i:expr, $state:expr) => {
        OperationsResult::$name(
            <use_cases::$path::ViewAndCacheType as ViewAndCache<Ch, $db>>::state_full_operation::<
                Id,
            >($i, $state)
            .await,
        )
    };
}

macro_rules! run_operation_check_apply {
    ($path:ident, $db:ty,$db_write:ty, $i:expr, $state:expr) => {
        let a = <use_cases::$path::ViewAndCacheType as ViewAndCache<Ch,$db>>::state_full_operation::<Id>($i, $state).await;
        let resources = <use_cases::$path::ViewAndCacheType as ViewAndCache<Ch,$db>>::extract_resource(&a);
        $db_write::write($state,&resources).await;
    };
}

macro_rules! get_user_uuid {
    ($path:ident, $db:ty, $i:expr) => {
        <use_cases::$path::ViewAndCacheType as ViewAndCache<Ch, $db>>::user_uuid(&$i)
    };
}

impl OperationsInput {
    pub(crate) async fn run_operation_check<Id: RowId, Ti: Time, Ch: Cache, Dbb: DbBundle<Ch>>(
        &self,
        state: &mut Ch,
    ) -> OperationsResult {
        match self {
            OperationsInput::SignUp(i) => {
                run_operation_check!(sign_up, SignUp, Dbb::SignUp, i, state)
            }
            OperationsInput::SignIn(i) => {
                run_operation_check!(sign_in, SignIn, Dbb::SignIn, i, state)
            }
            OperationsInput::CreateCompany(i) => {
                run_operation_check!(create_company, CreateCompany, Dbb::CreateCompany, i, state)
            }
            OperationsInput::CreateCompanyBranch(i) => {
                run_operation_check!(
                    create_company_branch,
                    CreateCompanyBranch,
                    Dbb::CreateCompanyBranch,
                    i,
                    state
                )
            }
            OperationsInput::ListCompanyAndBranch(i) => {
                run_operation_check!(
                    list_company_and_branch,
                    ListCompanyAndBranch,
                    Dbb::ListCompanyAndBranch,
                    i,
                    state
                )
            }
            OperationsInput::CreateAccount(i) => {
                run_operation_check!(create_account, CreateAccount, Dbb::CreateAccount, i, state)
            }
            OperationsInput::GetAllAccounts(_) => {
                unreachable!()
            }
            OperationsInput::GetAllAccountsForBranch(i) => {
                OperationsResult::GetAllAccountsForBranch(
                    <fetches::get_all_accounts_for_branch::ViewAndCacheType as ViewAndCache<
                        Ch,
                        Dbb::GetAllAccountsForBranch,
                    >>::state_full_operation::<Id>(i, state)
                    .await,
                )
            }
            OperationsInput::CreateAccountForBranch(i) => {
                run_operation_check!(
                    create_account_for_branch,
                    CreateAccountForBranch,
                    Dbb::CreateAccountForBranch,
                    i,
                    state
                )
            }
            OperationsInput::CreateJournalEntry(i) => {
                OperationsResult::CreateJournalEntry(
                    <use_cases::create_journal_entry::ViewAndCacheType<Ti> as ViewAndCache<
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
        Ti: Time,
        Ch: Cache,
        Dbb: DbBundle<Ch>,
    >(
        &self,
        state: &mut Ch,
    ) {
        match self {
            OperationsInput::SignUp(i) => {
                run_operation_check_apply!(sign_up, Dbb::SignUp, Dbb::WriteSignUp, i, state);
            }
            OperationsInput::SignIn(i) => {
                run_operation_check_apply!(sign_in, Dbb::SignIn, Dbb::WriteSignIn, i, state);
            }
            OperationsInput::CreateCompany(i) => {
                run_operation_check_apply!(
                    create_company,
                    Dbb::CreateCompany,
                    Dbb::WriteCreateCompany,
                    i,
                    state
                );
            }
            OperationsInput::CreateCompanyBranch(i) => {
                run_operation_check_apply!(
                    create_company_branch,
                    Dbb::CreateCompanyBranch,
                    Dbb::WriteCreateCompanyBranch,
                    i,
                    state
                );
            }
            OperationsInput::ListCompanyAndBranch(i) => {
                run_operation_check_apply!(
                    list_company_and_branch,
                    Dbb::ListCompanyAndBranch,
                    Dbb::WriteListCompanyAndBranch,
                    i,
                    state
                );
            }
            OperationsInput::CreateAccount(i) => {
                run_operation_check_apply!(
                    create_account,
                    Dbb::CreateAccount,
                    Dbb::WriteCreateAccount,
                    i,
                    state
                );
            }
            OperationsInput::GetAllAccounts(_) => {
                unreachable!()
            }
            OperationsInput::GetAllAccountsForBranch(_) => {
                unreachable!()
            }
            OperationsInput::CreateAccountForBranch(i) => {
                run_operation_check_apply!(
                    create_account_for_branch,
                    Dbb::CreateAccountForBranch,
                    Dbb::WriteCreateAccountForBranch,
                    i,
                    state
                );
            }
            OperationsInput::CreateJournalEntry(i) => {
                let result =
                    <use_cases::create_journal_entry::ViewAndCacheType<Ti> as ViewAndCache<
                        Ch,
                        Dbb::CreateJournalEntry,
                    >>::state_full_operation::<Id>(i, state)
                    .await;
                let resources =
                    <use_cases::create_journal_entry::ViewAndCacheType<Ti> as ViewAndCache<
                        Ch,
                        Dbb::CreateJournalEntry,
                    >>::store_resource(&result);
                state.write_resource_of_pending_txn(&resources.unwrap()).await;
            }
        }
    }

    pub(crate) fn get_user_uuid<Ti: Time, Ch: Cache, Dbb: DbBundle<Ch>>(
        &self,
    ) -> Option<&UuidType> {
        match self {
            OperationsInput::SignUp(i) => {
                get_user_uuid!(sign_up, Dbb::SignUp, i)
            }
            OperationsInput::SignIn(i) => {
                get_user_uuid!(sign_in, Dbb::SignIn, i)
            }
            OperationsInput::CreateCompany(i) => {
                get_user_uuid!(create_company, Dbb::CreateCompany, i)
            }
            OperationsInput::CreateCompanyBranch(i) => {
                get_user_uuid!(create_company_branch, Dbb::CreateCompanyBranch, i)
            }
            OperationsInput::ListCompanyAndBranch(i) => {
                get_user_uuid!(list_company_and_branch, Dbb::ListCompanyAndBranch, i)
            }
            OperationsInput::CreateAccount(i) => {
                get_user_uuid!(create_account, Dbb::CreateAccount, i)
            }
            OperationsInput::GetAllAccounts(i) => {
                fetches::get_all_accounts::ViewAndCacheType::user_uuid(i)
            }
            OperationsInput::GetAllAccountsForBranch(i) => {
                <fetches::get_all_accounts_for_branch::ViewAndCacheType as ViewAndCache<
                    Ch,
                    Dbb::GetAllAccountsForBranch,
                >>::user_uuid(i)
            }
            OperationsInput::CreateAccountForBranch(i) => {
                get_user_uuid!(create_account_for_branch, Dbb::CreateAccountForBranch, i)
            }
            OperationsInput::CreateJournalEntry(i) => {
                <use_cases::create_journal_entry::ViewAndCacheType<Ti> as ViewAndCache<
                    Ch,
                    Dbb::CreateJournalEntry,
                >>::user_uuid(i)
            }
        }
    }
}

macro_rules! extract_resource {
    ($path:ident, $db:ty, $i:expr) => {
        <use_cases::$path::ViewAndCacheType as ViewAndCache<Ch, $db>>::extract_resource($i)
    };
}

impl OperationsResult {
    pub(crate) fn extract_resource<Ti: Time, Ch: Cache, Dbb: DbBundle<Ch>>(
        &self,
    ) -> Vec<resource_utils::ResourceInfo> {
        match self {
            OperationsResult::SignIn(i) => {
                extract_resource!(sign_in, Dbb::SignIn, i)
            }
            OperationsResult::SignUp(i) => {
                extract_resource!(sign_up, Dbb::SignUp, i)
            }
            OperationsResult::CreateCompany(i) => {
                extract_resource!(create_company, Dbb::CreateCompany, i)
            }
            OperationsResult::CreateCompanyBranch(i) => {
                extract_resource!(create_company_branch, Dbb::CreateCompanyBranch, i)
            }
            OperationsResult::ListCompanyAndBranch(i) => {
                extract_resource!(list_company_and_branch, Dbb::ListCompanyAndBranch, i)
            }
            OperationsResult::CreateAccount(i) => {
                extract_resource!(create_account, Dbb::CreateAccount, i)
            }
            OperationsResult::GetAllAccounts(i) => {
                fetches::get_all_accounts::ViewAndCacheType::extract_resource(i)
            }
            OperationsResult::GetAllAccountsForBranch(i) => {
                <fetches::get_all_accounts_for_branch::ViewAndCacheType as ViewAndCache<
                    Ch,
                    Dbb::GetAllAccountsForBranch,
                >>::store_resource(i)
            }
            OperationsResult::CreateAccountForBranch(i) => {
                extract_resource!(create_account_for_branch, Dbb::CreateAccountForBranch, i)
            }
            OperationsResult::CreateJournalEntry(i) => {
                <use_cases::create_journal_entry::ViewAndCacheType<Ti> as ViewAndCache<
                    Ch,
                    Dbb::CreateJournalEntry,
                >>::store_resource(i)
            }
        }
    }

    pub(crate) fn is_ok(&self) -> bool {
        match self {
            OperationsResult::SignUp(i) => i.is_ok(),
            OperationsResult::SignIn(i) => i.is_ok(),
            OperationsResult::CreateCompany(i) => i.is_ok(),
            OperationsResult::CreateCompanyBranch(i) => i.is_ok(),
            OperationsResult::ListCompanyAndBranch(i) => i.is_ok(),
            OperationsResult::CreateAccount(i) => i.is_ok(),
            OperationsResult::GetAllAccounts(i) => i.is_ok(),
            OperationsResult::CreateAccountForBranch(i) => i.is_ok(),
            OperationsResult::GetAllAccountsForBranch(i) => i.is_ok(),
            OperationsResult::CreateJournalEntry(i) => i.is_ok(),
        }
    }
}
