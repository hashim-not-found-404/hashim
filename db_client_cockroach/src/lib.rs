pub mod db;
pub mod db_client;
pub mod db_transaction;

pub mod prelude {
    pub use crate::{db, db_client};

    pub(crate) use crate::{MyUuidConverter, MyUuidConverter1, db_transaction};
    // my crates
    pub(crate) use my_core::prelude::*;
}

use crate::prelude::*;
use uuid::Uuid;

pub trait MyUuidConverter {
    fn to_externel_uuid(&self) -> Uuid;
}

impl MyUuidConverter for db_types::UuidType {
    fn to_externel_uuid(&self) -> Uuid {
        Uuid::from_bytes(self.0) // assuming self.0 is [u8; 16]
    }
}

pub trait MyUuidConverter1 {
    fn to_uuid(self) -> db_types::UuidType;
}

impl MyUuidConverter1 for Uuid {
    fn to_uuid(self) -> db_types::UuidType {
        db_types::UuidType(*self.as_bytes())
    }
}
