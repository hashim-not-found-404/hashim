use crate::utility::utils;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Display, str::FromStr};

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Hash, Eq)]
pub struct UuidType(pub [u8; 16]);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JsonWebTokenType(pub String);

pub type ListOfCompanies = Vec<Company>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Company {
    pub uuid: UuidType,
    pub name: String,
    pub role: Role,
    pub branches: Vec<Branch>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Branch {
    pub uuid: UuidType,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

impl Location {
    pub fn is_valid(&self) -> bool {
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
    type Err = utils::DynamicError;

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
    type Err = utils::DynamicError;

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

    pub fn has_any(user_roles: &Vec<Self>, roles: &[Role]) -> bool {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Subscribe {
    TableUserFieldName,
    TableUserFieldId,
    TableCompanyFieldName,
    TableCompanyFieldCurrency,
    TableCompanyBranchFieldName,
    TableCompanyBranchFieldCompanyBelong,
    TableCompanyBranchFieldLocation,
    TableCompanyBranchFieldCurrency,
    TableAccessControlForCompanyFieldRole,
    TableAccessControlForCompanyFieldUser,
    TableAccessControlForCompanyFieldDataGroup,
    TableAccessControlForCompanyBranchFieldRole,
    TableAccessControlForCompanyBranchFieldUser,
    TableAccessControlForCompanyBranchFieldDataGroup,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Resource {
    Jwt(JsonWebTokenType),

    TableUserFieldName(String),
    TableUserFieldId(String),
    TableCompanyFieldName(String),
    TableCompanyFieldCurrency(Currency),
    TableCompanyBranchFieldName(String),
    TableCompanyBranchFieldCompanyBelong(UuidType),
    TableCompanyBranchFieldLocation(Location),
    TableCompanyBranchFieldCurrency(Currency),
    TableAccessControlForCompanyFieldRole(Role),
    TableAccessControlForCompanyFieldUser(UuidType),
    TableAccessControlForCompanyFieldDataGroup(UuidType),
    TableAccessControlForCompanyBranchFieldRole(Role),
    TableAccessControlForCompanyBranchFieldUser(UuidType),
    TableAccessControlForCompanyBranchFieldDataGroup(UuidType),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResourceInfo {
    pub row_uuid: UuidType,
    pub resource: Resource,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum UserUuidError {
    Invalid,
    NotAuthenticated,
    YouDontHavePermissionToDoThat,
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
