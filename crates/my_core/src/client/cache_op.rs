use crate::client;
use crate::client::fetches;
use crate::client::utility::cache::Cache;
use crate::domain::request_response;
use crate::domain::request_response::OperationsInput;
use crate::domain::request_response::OperationsOk;
use crate::domain::request_response::OperationsResult;
use crate::domain::use_cases;
use crate::domain::utility::types::DatabaseWrite;
use crate::domain::utility::types::RowId;
use crate::domain::utility::uuid::User;
use crate::utility::traits::Time;

pub trait DbBundle<Ch: Cache>: 'static {
    type ReadCreateAccount: for<'a> use_cases::create_account::DatabaseRead<Db<'a> = Ch>;
    type WriteCreateAccount: for<'a> DatabaseWrite<Db<'a> = Ch, Input = use_cases::create_account::Ok>;

    type ReadCreateAccountForBranch: for<'a> use_cases::create_account_for_branch::DatabaseRead<Db<'a> = Ch>;
    type WriteCreateAccountForBranch: for<'a> DatabaseWrite<Db<'a> = Ch, Input = use_cases::create_account_for_branch::Ok>;

    type ReadCreateJournalEntry: for<'a> use_cases::create_journal_entry::DatabaseRead<Db<'a> = Ch>;
    type WriteCreateJournalEntry: for<'a> DatabaseWrite<Db<'a> = Ch, Input = use_cases::create_journal_entry::Ok>;

    type ReadGetAllAccounts: for<'a> use_cases::get_all_accounts::DatabaseRead<Db<'a> = Ch>;
    type WriteGetAllAccounts: for<'a> DatabaseWrite<Db<'a> = Ch, Input = use_cases::get_all_accounts::Ok>;

    type ReadGetAllAccountsForBranch: for<'a> use_cases::get_all_accounts_for_branch::DatabaseRead<Db<'a> = Ch>;
    type WriteGetAllAccountsForBranch: for<'a> DatabaseWrite<Db<'a> = Ch, Input = use_cases::get_all_accounts_for_branch::Ok>;

    type ReadCreateCompany: for<'a> use_cases::create_company::DatabaseRead<Db<'a> = Ch>;
    type WriteCreateCompany: for<'a> DatabaseWrite<Db<'a> = Ch, Input = use_cases::create_company::Ok>;

    type ReadCreateCompanyBranch: for<'a> use_cases::create_company_branch::DatabaseRead<Db<'a> = Ch>;
    type WriteCreateCompanyBranch: for<'a> DatabaseWrite<Db<'a> = Ch, Input = use_cases::create_company_branch::Ok>;

    type ReadGetCompaniesAndBranches: for<'a> use_cases::get_companies_and_branches::DatabaseRead<Db<'a> = Ch>
        + 'static;
    type WriteGetCompaniesAndBranches: for<'a> DatabaseWrite<Db<'a> = Ch, Input = use_cases::get_companies_and_branches::Ok>
        + 'static;

    type ReadSignIn: for<'a> use_cases::sign_in::DatabaseRead<Db<'a> = Ch>;
    type WriteSignIn: for<'a> DatabaseWrite<Db<'a> = Ch, Input = use_cases::sign_in::Ok>;

    type ReadSignUp: for<'a> use_cases::sign_up::DatabaseRead<Db<'a> = Ch>;
    type WriteSignUp: for<'a> DatabaseWrite<Db<'a> = Ch, Input = use_cases::sign_up::Ok>;
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
        request_response::OperationsResult::$name(
            client::use_cases::$path::state_full_operation::<Ch, $db>($i, $state).await,
        )
    };
}

impl OperationsInput {
    pub(crate) async fn run_operation_check<Id: RowId, Ti: Time, Ch: Cache, Dbb: DbBundle<Ch>>(
        &self,
        state: &mut Ch,
    ) -> OperationsResult {
        match self {
            OperationsInput::SignUp(i) => {
                OperationsResult::SignUp(
                    client::use_cases::sign_up::state_full_operation::<Ch, Dbb::ReadSignUp>(
                        i, state,
                    )
                    .await,
                )
            }
            OperationsInput::SignIn(i) => {
                OperationsResult::SignIn(
                    client::use_cases::sign_in::state_full_operation::<Ch, Dbb::ReadSignIn>(
                        i, state,
                    )
                    .await,
                )
            }
            OperationsInput::CreateCompany(i) => {
                OperationsResult::CreateCompany(
                    client::use_cases::create_company::state_full_operation(i).await,
                )
            }
            OperationsInput::CreateCompanyBranch(i) => {
                OperationsResult::CreateCompanyBranch(
                    client::use_cases::create_company_branch::state_full_operation::<
                        Ch,
                        Dbb::ReadCreateCompanyBranch,
                    >(i, state)
                    .await,
                )
            }
            OperationsInput::GetCompaniesAndBranches(i) => {
                OperationsResult::GetCompaniesAndBranches(
                    client::use_cases::get_companies_and_branches::state_full_operation::<
                        Ch,
                        Dbb::ReadGetCompaniesAndBranches,
                    >(i, state)
                    .await,
                )
            }
            OperationsInput::CreateAccount(i) => {
                OperationsResult::CreateAccount(
                    client::use_cases::create_account::state_full_operation::<
                        Ch,
                        Dbb::ReadCreateAccount,
                    >(i, state)
                    .await,
                )
            }
            OperationsInput::GetAllAccounts(_) => {
                unreachable!()
            }
            OperationsInput::GetAllAccountsForBranch(i) => {
                OperationsResult::GetAllAccountsForBranch(
                    fetches::get_all_accounts_for_branch::state_full_operation::<
                        Ch,
                        Dbb::ReadGetAllAccountsForBranch,
                    >(i, state)
                    .await,
                )
            }
            OperationsInput::CreateAccountForBranch(i) => {
                run_operation_check!(
                    create_account_for_branch,
                    CreateAccountForBranch,
                    Dbb::ReadCreateAccountForBranch,
                    i,
                    state
                )
            }
            OperationsInput::CreateJournalEntry(i) => {
                OperationsResult::CreateJournalEntry(
                    client::use_cases::create_journal_entry::state_full_operation::<
                        Ti,
                        Ch,
                        Dbb::ReadCreateJournalEntry,
                    >(i, state)
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
                if let Ok(resources) = client::use_cases::sign_up::state_full_operation::<
                    Ch,
                    Dbb::ReadSignUp,
                >(i, state)
                .await
                {
                    Dbb::WriteSignUp::write(state, &resources).await.unwrap();
                };
            }
            OperationsInput::SignIn(i) => {
                if let Ok(resources) = client::use_cases::sign_in::state_full_operation::<
                    Ch,
                    Dbb::ReadSignIn,
                >(i, state)
                .await
                {
                    Dbb::WriteSignIn::write(state, &resources).await.unwrap();
                };
            }
            OperationsInput::CreateCompany(i) => {
                if let Ok(resources) =
                    client::use_cases::create_company::state_full_operation(i).await
                {
                    Dbb::WriteCreateCompany::write(state, &resources).await.unwrap();
                };
            }
            OperationsInput::CreateCompanyBranch(i) => {
                if let Ok(resources) =
                    client::use_cases::create_company_branch::state_full_operation::<
                        Ch,
                        Dbb::ReadCreateCompanyBranch,
                    >(i, state)
                    .await
                {
                    Dbb::WriteCreateCompanyBranch::write(state, &resources).await.unwrap();
                };
            }
            OperationsInput::GetCompaniesAndBranches(i) => {
                if let Ok(resources) =
                    client::use_cases::get_companies_and_branches::state_full_operation::<
                        Ch,
                        Dbb::ReadGetCompaniesAndBranches,
                    >(i, state)
                    .await
                {
                    Dbb::WriteGetCompaniesAndBranches::write(state, &resources).await.unwrap();
                };
            }
            OperationsInput::CreateAccount(i) => {
                if let Ok(resources) = client::use_cases::create_account::state_full_operation::<
                    Ch,
                    Dbb::ReadCreateAccount,
                >(i, state)
                .await
                {
                    Dbb::WriteCreateAccount::write(state, &resources).await.unwrap();
                };
            }
            OperationsInput::GetAllAccounts(_) => {
                unreachable!()
            }
            OperationsInput::GetAllAccountsForBranch(_) => {
                unreachable!()
            }
            OperationsInput::CreateAccountForBranch(i) => {
                if let Ok(resources) =
                    client::use_cases::create_account_for_branch::state_full_operation::<
                        Ch,
                        Dbb::ReadCreateAccountForBranch,
                    >(i, state)
                    .await
                {
                    Dbb::WriteCreateAccountForBranch::write(state, &resources).await.unwrap();
                };
            }
            OperationsInput::CreateJournalEntry(i) => {
                if let Ok(resources) =
                    client::use_cases::create_journal_entry::state_full_operation::<
                        Ti,
                        Ch,
                        Dbb::ReadCreateJournalEntry,
                    >(i, state)
                    .await
                {
                    Dbb::WriteCreateJournalEntry::write(state, &resources).await.unwrap();
                }
            }
        }
    }

    pub(crate) fn get_user_uuid<Ti: Time, Ch: Cache, Dbb: DbBundle<Ch>>(&self) -> Option<&User> {
        let user_uuid = match self {
            OperationsInput::SignUp(i) => &i.user_uuid,
            OperationsInput::SignIn(_) => return None,
            OperationsInput::CreateCompany(i) => &i.user_uuid,
            OperationsInput::CreateCompanyBranch(i) => &i.user_uuid,
            OperationsInput::GetCompaniesAndBranches(i) => &i.user_uuid,
            OperationsInput::CreateAccount(i) => &i.user_uuid,
            OperationsInput::GetAllAccounts(i) => &i.user_uuid,
            OperationsInput::GetAllAccountsForBranch(i) => &i.user_uuid,
            OperationsInput::CreateAccountForBranch(i) => &i.user_uuid,
            OperationsInput::CreateJournalEntry(i) => &i.user_uuid,
        };
        Some(user_uuid)
    }
}

impl OperationsResult {
    pub(crate) fn extract_resource(&self) -> Option<OperationsOk> {
        match self {
            OperationsResult::SignUp(i) => Some(OperationsOk::SignUp(i.clone().ok()?)),
            OperationsResult::SignIn(i) => Some(OperationsOk::SignIn(i.clone().ok()?)),
            OperationsResult::CreateCompany(i) => {
                Some(OperationsOk::CreateCompany(i.clone().ok()?))
            }
            OperationsResult::CreateCompanyBranch(i) => {
                Some(OperationsOk::CreateCompanyBranch(i.clone().ok()?))
            }
            OperationsResult::CreateAccount(i) => {
                Some(OperationsOk::CreateAccount(i.clone().ok()?))
            }
            OperationsResult::CreateAccountForBranch(i) => {
                Some(OperationsOk::CreateAccountForBranch(i.clone().ok()?))
            }
            OperationsResult::CreateJournalEntry(i) => {
                Some(OperationsOk::CreateJournalEntry(i.clone().ok()?))
            }
            OperationsResult::GetCompaniesAndBranches(i) => {
                Some(OperationsOk::GetCompaniesAndBranches(i.clone().ok()?))
            }
            OperationsResult::GetAllAccounts(i) => {
                Some(OperationsOk::GetAllAccounts(i.clone().ok()?))
            }
            OperationsResult::GetAllAccountsForBranch(i) => {
                Some(OperationsOk::GetAllAccountsForBranch(i.clone().ok()?))
            }
        }
    }

    pub(crate) fn is_ok(&self) -> bool {
        match self {
            OperationsResult::SignUp(i) => i.is_ok(),
            OperationsResult::SignIn(i) => i.is_ok(),
            OperationsResult::CreateCompany(i) => i.is_ok(),
            OperationsResult::CreateCompanyBranch(i) => i.is_ok(),
            OperationsResult::GetCompaniesAndBranches(i) => i.is_ok(),
            OperationsResult::CreateAccount(i) => i.is_ok(),
            OperationsResult::GetAllAccounts(i) => i.is_ok(),
            OperationsResult::CreateAccountForBranch(i) => i.is_ok(),
            OperationsResult::GetAllAccountsForBranch(i) => i.is_ok(),
            OperationsResult::CreateJournalEntry(i) => i.is_ok(),
        }
    }
}

pub(crate) async fn write_resource_to_cache<Ch: Cache, Dbb: DbBundle<Ch>>(
    cache: &mut Ch,
    resource: &OperationsOk,
) {
    match resource {
        OperationsOk::SignUp(i) => Dbb::WriteSignUp::write(cache, &i).await.unwrap(),
        OperationsOk::SignIn(i) => Dbb::WriteSignIn::write(cache, &i).await.unwrap(),
        OperationsOk::CreateCompany(i) => Dbb::WriteCreateCompany::write(cache, &i).await.unwrap(),
        OperationsOk::CreateCompanyBranch(i) => {
            Dbb::WriteCreateCompanyBranch::write(cache, &i).await.unwrap()
        }
        OperationsOk::CreateAccount(i) => Dbb::WriteCreateAccount::write(cache, &i).await.unwrap(),
        OperationsOk::CreateAccountForBranch(i) => {
            Dbb::WriteCreateAccountForBranch::write(cache, &i).await.unwrap()
        }
        OperationsOk::CreateJournalEntry(i) => {
            Dbb::WriteCreateJournalEntry::write(cache, &i).await.unwrap()
        }
        OperationsOk::GetCompaniesAndBranches(i) => {
            Dbb::WriteGetCompaniesAndBranches::write(cache, &i).await.unwrap()
        }
        OperationsOk::GetAllAccounts(i) => {
            Dbb::WriteGetAllAccounts::write(cache, &i).await.unwrap()
        }
        OperationsOk::GetAllAccountsForBranch(i) => {
            Dbb::WriteGetAllAccountsForBranch::write(cache, &i).await.unwrap()
        }
    }
}
