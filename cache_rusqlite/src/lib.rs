pub mod cache_adapter;

pub mod prelude {
    pub use crate::cache_adapter;

    pub(crate) use crate::{MyUuidConverter, MyUuidConverter1};
    pub(crate) use adapters::prelude::*;
    pub(crate) use my_core::prelude::*;
}

use crate::prelude::*;
use uuid::Uuid;

pub trait MyUuidConverter {
    fn to_string(&self) -> String;
}

impl MyUuidConverter for db_types::UuidType {
    fn to_string(&self) -> String {
        // Convert [u8; 16] → Uuid → String
        let uuid = Uuid::from_bytes(self.0);
        uuid.to_string()
    }
}

pub trait MyUuidConverter1 {
    fn to_uuid(self) -> db_types::UuidType;
}

impl MyUuidConverter1 for String {
    fn to_uuid(self) -> db_types::UuidType {
        // Parse string → Uuid → [u8; 16]
        let uuid = Uuid::parse_str(&self).expect("Invalid UUID string");
        db_types::UuidType(*uuid.as_bytes())
    }
}
