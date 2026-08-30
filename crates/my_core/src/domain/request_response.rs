use crate::domain::use_cases;
use crate::domain::utility::new_types::JsonWebTokenType;
use crate::domain::utility::new_types::NonceUuid;
use crate::domain::utility::types::HashimError;
use crate::domain::utility::types::JWTError;
use crate::domain::utility::types::NonceError;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Input {
    pub(crate) jwts:       Vec<JsonWebTokenType>,
    pub(crate) nonce:      NonceUuid,
    pub(crate) operations: Vec<Txn<OperationsInput>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct MyResult {
    pub(crate) jwts:       Vec<Result<(), JWTError>>,
    pub(crate) nonce:      Result<(), NonceError>,
    pub(crate) operations: Vec<Txn<OperationsResult>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Txn<T> {
    pub txn_number: u64,
    pub operation:  T,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OperationsInput {
    // auth
    SignUp(use_cases::sign_up::Input),
    SignIn(use_cases::sign_in::Input),
    // write
    CreateCompany(use_cases::create_company::Input),
    CreateCompanyBranch(use_cases::create_company_branch::Input),
    CreateAccount(use_cases::create_account::Input),
    CreateAccountForBranch(use_cases::create_account_for_branch::Input),
    CreateJournalEntry(use_cases::create_journal_entry::Input),
    // read
    GetCompaniesAndBranches(use_cases::get_companies_and_branches::Input),
    GetAllAccounts(use_cases::get_all_accounts::Input),
    GetAllAccountsForBranch(use_cases::get_all_accounts_for_branch::Input),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OperationsOk {
    // auth
    SignUp(use_cases::sign_up::Ok),
    SignIn(use_cases::sign_in::Ok),
    // write
    CreateCompany(use_cases::create_company::Ok),
    CreateCompanyBranch(use_cases::create_company_branch::Ok),
    CreateAccount(use_cases::create_account::Ok),
    CreateAccountForBranch(use_cases::create_account_for_branch::Ok),
    CreateJournalEntry(use_cases::create_journal_entry::Ok),
    // read
    GetCompaniesAndBranches(use_cases::get_companies_and_branches::Ok),
    GetAllAccounts(use_cases::get_all_accounts::Ok),
    GetAllAccountsForBranch(use_cases::get_all_accounts_for_branch::Ok),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OperationsResult {
    // auth
    SignUp(use_cases::sign_up::MyResult),
    SignIn(use_cases::sign_in::MyResult),
    // write
    CreateCompany(use_cases::create_company::MyResult),
    CreateCompanyBranch(use_cases::create_company_branch::MyResult),
    CreateAccount(use_cases::create_account::MyResult),
    CreateAccountForBranch(use_cases::create_account_for_branch::MyResult),
    CreateJournalEntry(use_cases::create_journal_entry::MyResult),
    // read
    GetCompaniesAndBranches(use_cases::get_companies_and_branches::MyResult),
    GetAllAccounts(use_cases::get_all_accounts::MyResult),
    GetAllAccountsForBranch(use_cases::get_all_accounts_for_branch::MyResult),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) enum ResourceDTO {
    CreateCompany(use_cases::create_company::Ok),
    CreateCompanyBranch(use_cases::create_company_branch::Ok),
    CreateAccount(use_cases::create_account::Ok),
    CreateAccountForBranch(use_cases::create_account_for_branch::Ok),
    CreateJournalEntry(use_cases::create_journal_entry::Ok),
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) enum FromServer {
    Error(HashimError),
    PushData(MyResult),
    Resources(Vec<ResourceDTO>),
}

pub(crate) type FromClient = Input;
