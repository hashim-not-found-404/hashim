use crate::accounting_domain::{db_types, decider};
use serde::{Deserialize, Serialize};

pub mod messages {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    pub enum FromServer {
        Error(db_types::HashimError),
        PushData(push_data::MyResult),
        Resources(Vec<db_types::ResourceInfo>),
    }

    pub type FromClient = push_data::Input;
}

pub mod push_data {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Input {
        pub jwts: Vec<db_types::JsonWebTokenType>,
        pub nonce: db_types::UuidType,
        pub operations: Vec<Txn<OperationsInput>>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct MyResult {
        pub jwts: Vec<Result<(), db_types::JWTError>>,
        pub nonce: Result<(), db_types::NonceError>,
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
        SignUp(decider::sign_up::Input),
        SignIn(decider::sign_in::Input),
        // write
        CreateCompany(decider::create_company::Input),
        CreateCompanyBranch(decider::create_company_branch::Input),
        // read
        ListCompanyAndBranch(decider::list_company_and_branch::Input),
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub enum OperationsResult {
        // auth
        SignUp(decider::sign_up::MyResult),
        SignIn(decider::sign_in::MyResult),
        // write
        CreateCompany(decider::create_company::MyResult),
        CreateCompanyBranch(decider::create_company_branch::MyResult),
        // read
        ListCompanyAndBranch(decider::list_company_and_branch::MyResult),
    }
}
