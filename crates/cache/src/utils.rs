use anyhow::Result;
use kernel::new_types::UuidType;
use std::ops::Deref;
use uuid::Uuid;

pub trait MyUuidConverter {
    fn to_string(&self) -> String;
}

impl MyUuidConverter for UuidType {
    fn to_string(&self) -> String {
        let uuid = Uuid::from_bytes(*self.deref());
        uuid.to_string()
    }
}

pub trait MyUuidConverter1 {
    fn to_uuid(self) -> Result<UuidType>;
}

impl MyUuidConverter1 for String {
    fn to_uuid(self) -> Result<UuidType> {
        let uuid = Uuid::parse_str(&self)?;
        Ok(UuidType::from(*uuid.as_bytes()))
    }
}
