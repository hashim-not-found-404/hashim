use serde::{Deserialize, Serialize};

// #[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DataGroup<RowId> {
    Company(RowId),
    Branch(RowId),
}

pub type RowId = String;

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
