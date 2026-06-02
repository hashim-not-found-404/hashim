use crate::prelude::*;

pub const HOST: &str = "127.0.0.1";
pub const PORT: u16 = 8081;
pub const ADDRESS: &str = "127.0.0.1:8081";

// there should be no generic in all the below types

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum UserUuidError {
    Invalid,
    NotAuthenticated,
    YouDontHavePermissionToDoThat,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResourceInfo {
    pub uuid: db_types::RowIdType,
    pub resource: server_methods::Resource,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
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
        PushData(Result<push_data::Result, HashimError>),
        Resources(Vec<ResourceInfo>),
    }

    pub type FromClient = push_data::Input;
}

pub mod push_data {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Input {
        pub jwts: Vec<String>,
        pub nonce: db_types::RowIdType,
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

    #[derive(Debug, Deserialize, Serialize)]
    pub enum OperationsInput {
        // auth
        SignUp(sign_up::Input),
        SignIn(sign_in::Input),
        // write
        CreateCompany(create_company::Input),
        CreateCompanyBranch(create_company_branch::Input),
        // read
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub enum OperationsResult {
        // auth
        SignUp(sign_up::Result),
        SignIn(sign_in::Result),
        // write
        CreateCompany(create_company::Result),
        CreateCompanyBranch(create_company_branch::Result),
        // read
    }
}

pub mod sign_up {
    use super::*;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub new_uuid: db_types::RowIdType,
        pub name: Option<String>,
        pub user_id: String,
        pub password: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Ok;

    #[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
    pub struct Error {
        pub new_uuid: Option<RowIdError>,
        pub user_id: Option<UserIdError>,
        pub name: Option<String>,
    }

    pub type Result = StdResult<Ok, Error>;

    // utility types
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub enum UserIdError {
        Duplicated,
    }
}

pub mod sign_in {
    use super::*;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub user_id: db_types::RowIdType,
        pub password: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Ok;

    #[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
    pub struct Error {
        pub user_id: Option<UserIdError>,
        pub password: Option<PasswordError>,
    }

    pub type Result = StdResult<Ok, Error>;

    // utility types
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub enum UserIdError {
        NotExist,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub enum PasswordError {
        WrongPassword,
    }
}

pub mod create_company {
    use super::*;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub user_uuid: db_types::RowIdType,
        pub new_uuid: db_types::RowIdType,
        pub company_name: String,
        pub currency: db_types::Currency,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Ok;

    #[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
    pub struct Error {
        pub user_uuid: Option<UserUuidError>,
        pub new_uuid: Option<RowIdError>,
    }

    pub type Result = StdResult<Ok, Error>;
}

pub mod create_company_branch {
    use super::*;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub user_uuid: db_types::RowIdType,
        pub new_uuid: db_types::RowIdType,
        pub company_belong: db_types::RowIdType,
        pub branch_name: String,
        pub location: db_types::Location,
        pub currency: db_types::Currency,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Ok;

    #[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
    pub struct Error {
        pub user_uuid: Option<UserUuidError>,
        pub new_uuid: Option<RowIdError>,
        pub company_belong: Option<CompanyBelongError>,
        pub branch_name: Option<BranchNameError>,
        pub location: Option<LocationError>,
    }

    pub type Result = StdResult<Ok, Error>;

    // utility types
    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub enum CompanyBelongError {
        IdInWrongFormat,
        NotExist,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub enum BranchNameError {
        Duplicated,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub enum LocationError {
        Invalid,
    }
}
