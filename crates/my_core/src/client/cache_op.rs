use crate::client;
use crate::client::fetches;
use crate::client::utility::cache::Cache;
use crate::domain::request_response;
use crate::domain::request_response::OperationsInput;
use crate::domain::request_response::OperationsResult;
use crate::domain::use_cases;
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
            client::use_cases::$path::state_full_operation::<Ch, $db>($i, $state).await,
        )
    };
}

macro_rules! run_operation_check_apply {
    ($path:ident, $db:ty, $i:expr, $state:expr) => {
        let a = client::use_cases::$path::state_full_operation::<Ch, $db>($i, $state).await;
        let resources = client::use_cases::$path::extract_resource(&a);
        $state.write_resource_of_pending_txn(&resources).await;
    };
}

impl OperationsInput {
    pub(crate) async fn run_operation_check<
        Id: RowId,
        Ti: traits::Time,
        Ch: Cache,
        Dbb: DbBundle<Ch>,
    >(
        &self,
        state: &mut Ch,
    ) -> OperationsResult {
        match self {
            OperationsInput::SignUp(i) => {
                request_response::OperationsResult::SignUp(
                    client::use_cases::sign_up::state_full_operation::<Ch, Dbb::SignUp>(i, state)
                        .await,
                )
            }
            OperationsInput::SignIn(i) => {
                request_response::OperationsResult::SignIn(
                    client::use_cases::sign_in::state_full_operation::<Ch, Dbb::SignIn>(i, state)
                        .await,
                )
            }
            OperationsInput::CreateCompany(i) => {
                request_response::OperationsResult::CreateCompany(
                    client::use_cases::create_company::state_full_operation(i).await,
                )
            }
            OperationsInput::CreateCompanyBranch(i) => {
                request_response::OperationsResult::CreateCompanyBranch(
                    client::use_cases::create_company_branch::state_full_operation::<
                        Ch,
                        Dbb::CreateCompanyBranch,
                    >(i, state)
                    .await,
                )
            }
            OperationsInput::ListCompanyAndBranch(i) => {
                request_response::OperationsResult::ListCompanyAndBranch(
                    client::use_cases::list_company_and_branch::state_full_operation::<
                        Ch,
                        Dbb::ListCompanyAndBranch,
                    >(i, state)
                    .await,
                )
            }
            OperationsInput::CreateAccount(i) => request_response::OperationsResult::CreateAccount(
                client::use_cases::create_account::state_full_operation::<Ch, Dbb::CreateAccount>(
                    i, state,
                )
                .await,
            ),
            OperationsInput::GetAllAccounts(_) => {
                unreachable!()
            }
            OperationsInput::GetAllAccountsForBranch(i) => {
                OperationsResult::GetAllAccountsForBranch(
                    fetches::get_all_accounts_for_branch::state_full_operation::<
                        Ch,
                        Dbb::GetAllAccountsForBranch,
                    >(i, state)
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
                    client::use_cases::create_journal_entry::state_full_operation::<
                        Ti,
                        Ch,
                        Dbb::CreateJournalEntry,
                    >(i, state)
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
            OperationsInput::SignUp(i) => {
                run_operation_check_apply!(sign_up, Dbb::SignUp, i, state);
            }
            OperationsInput::SignIn(i) => {
                run_operation_check_apply!(sign_in, Dbb::SignIn, i, state);
            }
            OperationsInput::CreateCompany(i) => {
                let a = client::use_cases::create_company::state_full_operation(i).await;
                let resources = client::use_cases::create_company::extract_resource(&a);
                state.write_resource_of_pending_txn(&resources).await;
            }
            OperationsInput::CreateCompanyBranch(i) => {
                run_operation_check_apply!(
                    create_company_branch,
                    Dbb::CreateCompanyBranch,
                    i,
                    state
                );
            }
            OperationsInput::ListCompanyAndBranch(i) => {
                run_operation_check_apply!(
                    list_company_and_branch,
                    Dbb::ListCompanyAndBranch,
                    i,
                    state
                );
            }
            OperationsInput::CreateAccount(i) => {
                run_operation_check_apply!(create_account, Dbb::CreateAccount, i, state);
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
                    i,
                    state
                );
            }
            OperationsInput::CreateJournalEntry(i) => {
                let result = client::use_cases::create_journal_entry::state_full_operation::<
                    Ti,
                    Ch,
                    Dbb::CreateJournalEntry,
                >(i, state)
                .await;
                let resources = client::use_cases::create_journal_entry::extract_resource(&result);
                state.write_resource_of_pending_txn(&resources).await;
            }
        }
    }

    pub(crate) fn get_user_uuid<Ti: traits::Time, Ch: Cache, Dbb: DbBundle<Ch>>(
        &self,
    ) -> Option<&User> {
        match self {
            OperationsInput::SignUp(i) => client::use_cases::sign_up::user_uuid(i),
            OperationsInput::SignIn(i) => client::use_cases::sign_in::user_uuid(i),
            OperationsInput::CreateCompany(i) => client::use_cases::create_company::user_uuid(i),
            OperationsInput::CreateCompanyBranch(i) => {
                client::use_cases::create_company_branch::user_uuid(i)
            }
            OperationsInput::ListCompanyAndBranch(i) => {
                client::use_cases::list_company_and_branch::user_uuid(i)
            }
            OperationsInput::CreateAccount(i) => client::use_cases::create_account::user_uuid(i),
            OperationsInput::GetAllAccounts(i) => client::fetches::get_all_accounts::user_uuid(i),
            OperationsInput::GetAllAccountsForBranch(i) => {
                client::fetches::get_all_accounts_for_branch::user_uuid(i)
            }
            OperationsInput::CreateAccountForBranch(i) => {
                client::use_cases::create_account_for_branch::user_uuid(i)
            }
            OperationsInput::CreateJournalEntry(i) => {
                client::use_cases::create_journal_entry::user_uuid(i)
            }
        }
    }
}

impl OperationsResult {
    pub(crate) fn extract_resource(&self) -> Vec<resource_utils::ResourceInfo> {
        match self {
            OperationsResult::SignIn(i) => client::use_cases::sign_in::extract_resource(i),
            OperationsResult::SignUp(i) => client::use_cases::sign_up::extract_resource(i),
            OperationsResult::CreateCompany(i) => {
                client::use_cases::create_company::extract_resource(i)
            }
            OperationsResult::CreateCompanyBranch(i) => {
                client::use_cases::create_company_branch::extract_resource(i)
            }
            OperationsResult::ListCompanyAndBranch(i) => {
                client::use_cases::list_company_and_branch::extract_resource(i)
            }
            OperationsResult::CreateAccount(i) => {
                client::use_cases::create_account::extract_resource(i)
            }
            OperationsResult::GetAllAccounts(i) => fetches::get_all_accounts::extract_resource(i),
            OperationsResult::GetAllAccountsForBranch(i) => {
                fetches::get_all_accounts_for_branch::extract_resource(i)
            }
            OperationsResult::CreateAccountForBranch(i) => {
                client::use_cases::create_account_for_branch::extract_resource(i)
            }
            OperationsResult::CreateJournalEntry(i) => {
                client::use_cases::create_journal_entry::extract_resource(i)
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
