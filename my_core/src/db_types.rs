use crate::prelude::*;

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Hash, Eq)]
pub struct UuidType(pub [u8; 16]);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JsonWebTokenType(pub String);

pub type ListOfCompanies = Vec<db_types::Company>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Company {
    pub uuid: db_types::UuidType,
    pub name: String,
    pub role: db_types::Role,
    pub branches: Vec<Branch>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Branch {
    pub uuid: db_types::UuidType,
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
    type Err = DynamicError;

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
    type Err = DynamicError;

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
