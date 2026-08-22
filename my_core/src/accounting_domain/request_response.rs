use crate::accounting_domain::cases;
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types::HashimError;
use serde::Deserialize;
use serde::Serialize;

pub mod push_data {
    use super::Deserialize;
    use super::Serialize;
    use super::cases;
    use crate::accounting_domain::utility::types::JWTError;
    use crate::accounting_domain::utility::types::JsonWebTokenType;
    use crate::accounting_domain::utility::types::NonceError;
    use crate::accounting_domain::utility::types::UuidType;

    #[derive(Debug, Deserialize, Serialize)]
    pub(crate) struct Input {
        pub(crate) jwts:       Vec<JsonWebTokenType>,
        pub(crate) nonce:      UuidType,
        pub(crate) operations: Vec<Txn<OperationsInput>>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub(crate) struct MyResult {
        pub(crate) jwts:       Vec<Result<(), JWTError>>,
        pub(crate) nonce:      Result<(), NonceError>,
        pub(crate) operations: Vec<Txn<OperationsResult>>,
    }

    // utility types
    #[derive(Debug, Deserialize, Serialize)]
    pub struct Txn<T> {
        pub txn_number: u64,
        pub operation:  T,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum OperationsInput {
        // auth
        SignUp(cases::sign_up::Input),
        SignIn(cases::sign_in::Input),
        // write
        CreateCompany(cases::create_company::Input),
        CreateCompanyBranch(cases::create_company_branch::Input),
        CreateAccount(cases::create_account::Input),
        CreateAccountForBranch(cases::create_account_for_branch::Input),
        CreateJournalEntry(cases::create_journal_entry::Input),
        // read
        ListCompanyAndBranch(cases::list_company_and_branch::Input),
        GetAllAccounts(cases::get_all_accounts::Input),
        GetAllAccountsForBranch(cases::get_all_accounts_for_branch::Input),
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum OperationsResult {
        // auth
        SignUp(cases::sign_up::MyResult),
        SignIn(cases::sign_in::MyResult),
        // write
        CreateCompany(cases::create_company::MyResult),
        CreateCompanyBranch(cases::create_company_branch::MyResult),
        CreateAccount(cases::create_account::MyResult),
        CreateAccountForBranch(cases::create_account_for_branch::MyResult),
        CreateJournalEntry(cases::create_journal_entry::MyResult),
        // read
        ListCompanyAndBranch(cases::list_company_and_branch::MyResult),
        GetAllAccounts(cases::get_all_accounts::MyResult),
        GetAllAccountsForBranch(cases::get_all_accounts_for_branch::MyResult),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) enum FromServer {
    Error(HashimError),
    PushData(push_data::MyResult),
    Resources(Vec<resource_utils::ResourceInfo>),
}

pub(crate) type FromClient = push_data::Input;
