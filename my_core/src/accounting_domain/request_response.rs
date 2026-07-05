use crate::accounting_domain::{cases, types};
use serde::{Deserialize, Serialize};

pub mod messages {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    pub enum FromServer {
        Error(types::HashimError),
        PushData(push_data::MyResult),
        Resources(Vec<types::ResourceInfo>),
    }

    pub type FromClient = push_data::Input;
}

pub mod push_data {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Input {
        pub jwts: Vec<types::JsonWebTokenType>,
        pub nonce: types::UuidType,
        pub operations: Vec<Txn<OperationsInput>>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct MyResult {
        pub jwts: Vec<Result<(), types::JWTError>>,
        pub nonce: Result<(), types::NonceError>,
        pub operations: Vec<Txn<OperationsResult>>,
    }

    // utility types
    #[derive(Debug, Deserialize, Serialize)]
    pub struct Txn<T> {
        pub txn_number: u64,
        pub operation: T,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub enum OperationsInput {
        // auth
        SignUp(cases::sign_up::Input),
        SignIn(cases::sign_in::Input),
        // write
        CreateCompany(cases::create_company::Input),
        CreateCompanyBranch(cases::create_company_branch::Input),
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
        // read
        ListCompanyAndBranch(cases::list_company_and_branch::MyResult),
    }
}
