use serde::{Deserialize, Serialize};

pub const HOST: &str = "127.0.0.1";
pub const PORT: u16 = 8081;
pub const ADDRESS: &str = "127.0.0.1:8081";

pub mod custom_types {
    use crate::db_types;

    use super::*;

    pub type RowId = String;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Location {
        latitude: f64,
        longitude: f64,
    }

    #[derive(Debug, Deserialize, Serialize, Clone, Default)]
    pub enum Currency {
        #[default]
        USD,
        IQD,
    }
    impl Currency {
        pub fn as_str(&self) -> &str {
            match self {
                Self::IQD => "IQD",
                _ => todo!(),
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub enum Role {
        Manager,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Branch {
        pub name: String,
        pub location: custom_types::Location,
        pub currency: custom_types::Currency,
        pub role: Vec<Role>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Company {
        pub name: String,
        pub currency: custom_types::Currency,
        pub branches: Vec<Branch>,
        pub role: Vec<Role>,
    }
}

pub mod business_layer {
    use super::*;

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    pub enum OperationMode {
        CheckFromCache,
        CheckFromServer,
        SubmitToServer,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input<T> {
        pub transaction_number: u64,
        pub jwt: String,
        pub submit: OperationMode,
        pub content: T,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub enum Error<Input> {
        // here is the error that is made at the core
        DuplicateTransaction,
        InvalidJWT,
        DataHasBeenChangedByOthers,

        /// this contain the error of the input that user made
        InvalidInput(Input),
    }

    pub type Result<Ok, InputError, ExternalError> =
        std::result::Result<std::result::Result<Ok, Error<InputError>>, ExternalError>;
}

// there should be no generic in all the below types

pub mod sign_up {
    use super::*;

    pub const PATH: &str = "/sign_up";

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

    pub type Result<Ext> = business_layer::Result<Ok, Error, Ext>;
}

pub mod sign_in {
    use super::*;

    pub const PATH: &str = "/sign_in";

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub user_id: custom_types::RowId,
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

    pub type Result<Ext> = business_layer::Result<Ok, Error, Ext>;
}

pub mod get_all_user_roles {
    use super::*;

    pub const PATH: &str = "/get_all_user_roles";
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok {
        pub all_roles: Vec<custom_types::Company>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Error;

    pub type Result<Ext> = business_layer::Result<Ok, Error, Ext>;
}

pub mod create_company {
    use super::*;

    pub const PATH: &str = "/create_company";

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub name: String,
        pub currency: custom_types::Currency,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Ok;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Error;

    pub type Result<Ext> = business_layer::Result<Ok, Error, Ext>;
}

pub mod create_company_branch {
    use super::*;

    pub const PATH: &str = "/create_company_branch";

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Input {
        pub company_belong: custom_types::RowId,
        pub name: String,
        pub location: custom_types::Location,
        pub currency: custom_types::Currency,
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

    pub type Result<Ext> = business_layer::Result<Ok, Error, Ext>;
}
