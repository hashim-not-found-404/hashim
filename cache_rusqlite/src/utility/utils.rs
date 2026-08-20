use my_core::accounting_domain::utility::types;
use uuid::Uuid;

pub(crate) trait MyUuidConverter {
    fn to_string(&self) -> String;
}

impl MyUuidConverter for types::UuidType {
    fn to_string(&self) -> String {
        let uuid = Uuid::from_bytes(self.0);
        uuid.to_string()
    }
}

pub(crate) trait MyUuidConverter1 {
    fn to_uuid(self) -> types::UuidType;
}

impl MyUuidConverter1 for String {
    fn to_uuid(self) -> types::UuidType {
        let uuid = Uuid::parse_str(&self).unwrap();
        types::UuidType(*uuid.as_bytes())
    }
}
