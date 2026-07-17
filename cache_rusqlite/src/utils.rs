use my_core::accounting_domain::cases::utility::types;
use uuid::Uuid;

pub(crate) trait MyUuidConverter {
    fn to_string(&self) -> String;
}

impl MyUuidConverter for types::UuidType {
    fn to_string(&self) -> String {
        // Convert [u8; 16] → Uuid → String
        let uuid = Uuid::from_bytes(self.0);
        uuid.to_string()
    }
}

pub(crate) trait MyUuidConverter1 {
    fn to_uuid(self) -> types::UuidType;
}

impl MyUuidConverter1 for String {
    fn to_uuid(self) -> types::UuidType {
        // Parse string → Uuid → [u8; 16]
        let uuid = Uuid::parse_str(&self).expect("Invalid UUID string");
        types::UuidType(*uuid.as_bytes())
    }
}
