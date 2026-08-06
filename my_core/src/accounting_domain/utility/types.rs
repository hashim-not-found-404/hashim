use crate::utility::traits;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;

pub trait RowId: 'static {
    fn generate() -> UuidType;
    fn get_time_as_seconds(uuid: &UuidType) -> Option<u64>;
    fn validate(uuid: &UuidType) -> bool;
}

pub trait HashedPassword {
    fn sign_up(password: &String) -> String;
    fn sign_in(password: &String, password_hash: &String) -> bool;
}

pub trait JWT: 'static {
    fn new() -> Self;
    fn sign(&self, user_uuid: &UuidType) -> JsonWebTokenType;
    fn validate(&self, token: JsonWebTokenType) -> Option<UuidType>;
}

pub(crate) trait MyErrorTrait {
    fn is_there_error(&self) -> bool;
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct UuidType(pub [u8; 16]);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JsonWebTokenType(pub String);

pub type ListOfCompanies = Vec<Company>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Company {
    pub uuid:        UuidType,
    pub name:        String,
    pub(crate) role: Role,
    pub branches:    Vec<Branch>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Branch {
    pub uuid: UuidType,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Location {
    pub latitude:  f64,
    pub longitude: f64,
}

impl Location {
    pub(crate) fn is_valid(&self) -> bool {
        // Check bounds for latitude and longitude
        // Also ensure the values are finite (not NaN or Infinity)
        self.latitude >= -90.0
            && self.latitude <= 90.0
            && self.longitude >= -180.0
            && self.longitude <= 180.0
            && self.latitude.is_finite()
            && self.longitude.is_finite()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub enum Currency {
    #[default]
    USD,
    IQD,
}

impl FromStr for Currency {
    type Err = traits::DynamicError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "USD" => Ok(Currency::USD),
            "IQD" => Ok(Currency::IQD),
            _ => Err("not exist".into()),
        }
    }
}

impl Currency {
    pub fn as_str(&self) -> &str {
        match self {
            Currency::USD => "USD",
            Currency::IQD => "IQD",
        }
    }
}

#[derive(Default, Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum Role {
    #[default]
    Manager,
    CoManager,
}

impl FromStr for Role {
    type Err = traits::DynamicError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Manager" => Ok(Role::Manager),
            "CoManager" => Ok(Role::CoManager),
            _ => Err("not exist".into()),
        }
    }
}

impl Role {
    pub fn as_str(&self) -> &str {
        match self {
            Role::Manager => "Manager",
            Role::CoManager => "CoManager",
        }
    }

    pub(crate) fn has_any(user_roles: &Vec<Self>, roles: &[Role]) -> bool {
        for role in roles {
            if user_roles.contains(role) {
                return true;
            }
        }
        false
    }
}

pub const HOST: &str = "127.0.0.1";
pub const PORT: u16 = 8081;
pub const ADDRESS: &str = "127.0.0.1:8081";

// there should be no generic in all the below types
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub(crate) enum UserUuidError {
    Invalid,
    NotAuthenticated,
    YouDontHavePermissionToDoThat,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub(crate) enum RowIdError {
    Invalid,
    Duplicated,
    NotExist,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) enum NonceError {
    Invalid,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) enum JWTError {
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
