use crate::db_types;
use serde::{Deserialize, Serialize};

pub const HOST: &str = "127.0.0.1";
pub const PORT: u16 = 8081;
pub const ADDRESS: &str = "127.0.0.1:8081";

// there should be no generic in all the below types

pub mod sign_up {
    use super::*;
    pub const PATH: &str = "sign_up";

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub name: Option<String>,
        pub user_id: String,
        pub password: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok {
        pub jwt: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub enum UserIdError {
        Duplicated,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Error {
        pub user_id: Option<UserIdError>,
        pub name: Option<String>,
    }

    pub type Result = std::result::Result<Ok, Error>;
}

pub mod sign_in {
    use super::*;
    pub const PATH: &str = "sign_in";

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub user_id: db_types::RowId,
        pub password: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok {
        pub jwt: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub enum UserIdError {
        NotExist,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub enum PasswordError {
        WrongPassword,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Error {
        pub user_id: Option<UserIdError>,
        pub password: Option<PasswordError>,
    }

    pub type Result = std::result::Result<Ok, Error>;
}

pub mod get_all_user_roles {
    use super::*;
    pub const PATH: &str = "get_all_user_roles";

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok {
        pub all_roles: Vec<db_types::Company>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Error;

    pub type Result = std::result::Result<Ok, Error>;
}

pub mod create_company {
    use super::*;
    pub const PATH: &str = "create_company";

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub name: String,
        pub currency: db_types::Currency,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Error;

    pub type Result = std::result::Result<Ok, Error>;
}

pub mod create_company_branch {
    use super::*;
    pub const PATH: &str = "create_company_branch";

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub company_belong: db_types::RowId,
        pub name: String,
        pub location: db_types::Location,
        pub currency: db_types::Currency,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub enum CompanyError {
        IdInWrongFormat,
        NotExist,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub enum NameError {
        Duplicated,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub enum LocationError {
        NotExist,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Error {
        pub company_belong: Option<CompanyError>,
        pub name: Option<NameError>,
        pub location: Option<LocationError>,
    }

    pub type Result = std::result::Result<Ok, Error>;
}
