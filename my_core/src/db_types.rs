use crate::prelude::*;

#[derive(PartialEq, Clone, Hash, Eq, Debug, Deserialize, Serialize)]
pub struct UuidType(pub String);

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Location {
    latitude: f64,
    longitude: f64,
}

impl Location {
    pub fn is_valid(&self) -> bool {
        todo!();
    }
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
