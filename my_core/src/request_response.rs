use crate::prelude::*;

pub const HOST: &str = "127.0.0.1";
pub const PORT: u16 = 8081;
pub const ADDRESS: &str = "127.0.0.1:8081";

// there should be no generic in all the below types

#[derive(Debug, Deserialize, Serialize)]
pub struct ResourceInfo {
    pub version: u64,
    pub uuid: String,
    pub resource: server_methods::Resource,
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
    DecodingErrorAtServer,
    ConnectionClosed,
}

impl Error for HashimError {}

impl Display for HashimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashimError::InternalServerError => write!(f, "Internal Server Error"),
            HashimError::DecodingErrorAtServer => write!(f, "Decoding Error at Server"),
            HashimError::ConnectionClosed => write!(f, "Connection Closed"),
        }
    }
}

pub mod sign_up {
    use super::*;
    pub const PATH: &str = "sign_up";

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Input {
        pub name: Option<String>,
        pub user_id: String,
        pub password: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Ok {
        pub jwt: String,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub enum UserIdError {
        Duplicated,
    }

    #[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
    pub struct Error {
        pub user_id: Option<UserIdError>,
        pub name: Option<String>,
    }

    pub type Result = StdResult<Ok, Error>;
}

pub mod sign_in {
    use super::*;
    pub const PATH: &str = "sign_in";

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Input {
        pub user_id: db_types::RowIdType,
        pub password: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Ok {
        pub jwt: String,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub enum UserIdError {
        NotExist,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub enum PasswordError {
        WrongPassword,
    }

    #[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
    pub struct Error {
        pub user_id: Option<UserIdError>,
        pub password: Option<PasswordError>,
    }

    pub type Result = StdResult<Ok, Error>;
}

pub mod data_receiver {
    use super::*;
    pub const PATH: &str = "data_receiver";

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Input(Vec<ResourceInfo>);
}

pub mod create_company {
    use super::*;
    pub const PATH: &str = "create_company";

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Input {
        pub jwt: String,
        pub nonce: db_types::RowIdType,
        pub txn_number: u32,
        pub company_name: String,
        pub currency: db_types::Currency,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Ok {
        pub resources: Vec<ResourceInfo>,
    }

    #[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
    pub struct Error {
        pub jwt: Option<JWTError>,
        pub nonce: Option<NonceError>,
    }

    pub type Result = StdResult<Ok, Error>;
}

pub mod create_company_branch {
    use super::*;
    pub const PATH: &str = "create_company_branch";

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Input {
        pub jwt: String,
        pub nonce: db_types::RowIdType,
        pub txn_number: u32,
        pub company_belong: db_types::RowIdType,
        pub branch_name: String,
        pub location: db_types::Location,
        pub currency: db_types::Currency,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Ok {
        pub resources: Vec<ResourceInfo>,
    }

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

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub enum AuthorizationError {
        YouDontHavePermissionToDoThat,
    }

    #[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
    pub struct Error {
        pub jwt: Option<JWTError>,
        pub nonce: Option<NonceError>,
        pub authorization: Option<AuthorizationError>,
        pub company_belong: Option<CompanyBelongError>,
        pub branch_name: Option<BranchNameError>,
        pub location: Option<LocationError>,
    }

    pub type Result = StdResult<Ok, Error>;
}
