use crate::prelude::*;

pub const HOST: &str = "127.0.0.1";
pub const PORT: u16 = 8081;
pub const ADDRESS: &str = "127.0.0.1:8081";

// there should be no generic in all the below types

#[derive(Debug, Deserialize, Serialize)]
pub enum NouncError {
    AlreadyUsed,
}

#[derive(Debug, Deserialize, Serialize)]
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

    #[derive(Debug, Deserialize, Serialize)]
    pub enum UserIdError {
        Duplicated,
    }

    #[derive(Debug, Deserialize, Serialize)]
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

    #[derive(Debug, Deserialize, Serialize)]
    pub enum UserIdError {
        NotExist,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub enum PasswordError {
        WrongPassword,
    }

    #[derive(Debug, Deserialize, Serialize)]
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
    pub struct Content {
        pub table: String,
        pub column: String,
        pub uuid: String,
        pub value: String,
        pub version: u64,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Input(Vec<Content>);
}

pub mod get_all_user_roles {
    use super::*;
    pub const PATH: &str = "get_all_user_roles";

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Input;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Ok {
        pub all_roles: Vec<db_types::Company>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Error;

    pub type Result = StdResult<Ok, Error>;
}

pub mod create_company {
    use super::*;
    pub const PATH: &str = "create_company";

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Input {
        pub jwt: String,
        pub nounc: u64,
        pub company_name: String,
        pub currency: db_types::Currency,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Ok;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Error {
        pub jwt: Option<JWTError>,
        pub nounc: Option<NouncError>,
    }

    pub type Result = StdResult<Ok, Error>;
}

pub mod create_company_branch {
    use super::*;
    pub const PATH: &str = "create_company_branch";

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Input {
        pub company_belong: db_types::RowIdType,
        pub name: String,
        pub location: db_types::Location,
        pub currency: db_types::Currency,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Ok;

    #[derive(Debug, Deserialize, Serialize)]
    pub enum CompanyError {
        IdInWrongFormat,
        NotExist,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub enum NameError {
        Duplicated,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub enum LocationError {
        NotExist,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Error {
        pub company_belong: Option<CompanyError>,
        pub name: Option<NameError>,
        pub location: Option<LocationError>,
    }

    pub type Result = StdResult<Ok, Error>;
}
