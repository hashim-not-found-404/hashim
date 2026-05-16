use std::str::FromStr;

use crate::prelude::*;

// #[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DataGroup<RowId> {
    Company(RowId),
    Branch(RowId),
}

pub type RowIdType = String;

// maybe i will only check from cache by sync it with the server
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum OperationMode {
    CheckFromCache,
    SubmitToServer,
}

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

impl FromStr for Currency {
    type Err = DynamicError;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Role {
    Manager,
}

impl FromStr for Role {
    type Err = DynamicError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Manager" => Ok(Role::Manager),
            _ => Err("not exist".into()),
        }
    }
}

impl Role {
    pub fn as_str(&self) -> &str {
        match self {
            Role::Manager => "Manager",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Branch {
    pub name: String,
    pub location: Location,
    pub currency: Currency,
    pub role: Vec<Role>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Company {
    pub name: String,
    pub currency: Currency,
    pub branches: Vec<Branch>,
    pub role: Vec<Role>,
}
