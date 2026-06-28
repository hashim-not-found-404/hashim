use crate::prelude::*;
use std::result::Result as StdResult;

pub const HOST: &str = "127.0.0.1";
pub const PORT: u16 = 8081;
pub const ADDRESS: &str = "127.0.0.1:8081";

// there should be no generic in all the below types

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum UserUuidError {
    Invalid,
    NotAuthenticated,
    YouDontHavePermissionToDoThat,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResourceInfo {
    pub row_uuid: db_types::UuidType,
    pub resource: server_methods::Resource,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum RowIdError {
    Invalid,
    Duplicated,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum NonceError {
    Invalid,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum JWTError {
    Invalid,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum HashimError {
    InternalServerError,
    InvalidDataFormat,
    ConnectionClosed,
}

impl Error for HashimError {}

impl Display for HashimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashimError::InternalServerError => write!(f, "Internal Server Error"),
            HashimError::InvalidDataFormat => write!(f, "Invalid Data Format"),
            HashimError::ConnectionClosed => write!(f, "Connection Closed"),
        }
    }
}

pub mod messages {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    pub enum FromServer {
        Error(HashimError),
        PushData(push_data::Result),
        Resources(Vec<ResourceInfo>),
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
    pub struct Result {
        pub jwts: Vec<StdResult<(), JWTError>>,
        pub nonce: StdResult<(), NonceError>,
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
        SignUp(decider::sign_up::Result),
        SignIn(decider::sign_in::Result),
        // write
        CreateCompany(decider::create_company::Result),
        CreateCompanyBranch(decider::create_company_branch::Result),
        // read
        ListCompanyAndBranch(decider::list_company_and_branch::Result),
    }
}
