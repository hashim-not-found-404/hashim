use my_core::accounting_domain::types;
use uuid::Uuid;

pub(crate) trait MyUuidConverter {
    fn to_externel_uuid(&self) -> Uuid;
}

impl MyUuidConverter for types::UuidType {
    fn to_externel_uuid(&self) -> Uuid {
        Uuid::from_bytes(self.0) // assuming self.0 is [u8; 16]
    }
}

pub(crate) trait MyUuidConverter1 {
    fn to_uuid(self) -> types::UuidType;
}

impl MyUuidConverter1 for Uuid {
    fn to_uuid(self) -> types::UuidType {
        types::UuidType(*self.as_bytes())
    }
}
