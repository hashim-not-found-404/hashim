use crate::accounting_domain::cases;
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types;
use serde::Deserialize;
use serde::Serialize;

pub(crate) mod messages {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    pub(crate) enum FromServer {
        Error(types::HashimError),
        PushData(push_data::MyResult),
        Resources(Vec<resource_utils::ResourceInfo>),
    }

    pub(crate) type FromClient = push_data::Input;
}

pub mod push_data {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    pub(crate) struct Input {
        pub(crate) jwts:       Vec<types::JsonWebTokenType>,
        pub(crate) nonce:      types::UuidType,
        pub(crate) operations: Vec<Txn<OperationsInput>>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub(crate) struct MyResult {
        pub(crate) jwts:       Vec<Result<(), types::JWTError>>,
        pub(crate) nonce:      Result<(), types::NonceError>,
        pub(crate) operations: Vec<Txn<OperationsResult>>,
    }

    // utility types
    #[derive(Debug, Deserialize, Serialize)]
    pub struct Txn<T> {
        pub txn_number: u64,
        pub operation:  T,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub enum OperationsInput {
        // auth
        SignUp(cases::sign_up::Input),
        SignIn(cases::sign_in::Input),
        // write
        CreateCompany(cases::create_company::Input),
        CreateCompanyBranch(cases::create_company_branch::Input),
        CreateAccount(cases::create_account::Input),
        // read
        ListCompanyAndBranch(cases::list_company_and_branch::Input),
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub enum OperationsResult {
        // auth
        SignUp(cases::sign_up::MyResult),
        SignIn(cases::sign_in::MyResult),
        // write
        CreateCompany(cases::create_company::MyResult),
        CreateCompanyBranch(cases::create_company_branch::MyResult),
        CreateAccount(cases::create_account::MyResult),
        // read
        ListCompanyAndBranch(cases::list_company_and_branch::MyResult),
    }
}
