pub mod m {
    use crate::prelude::*;
    use uuid::Uuid;

    pub struct S;

    impl RowId for S {
        fn generate() -> db_types::UuidType {
            // Generate a random UUID v4
            db_types::UuidType(*Uuid::now_v7().as_bytes())
        }

        fn get_time_as_seconds(uuid: &db_types::UuidType) -> u64 {
            // Convert bytes to Uuid and extract timestamp (for UUID v7)
            let u = Uuid::from_bytes(uuid.0);
            if let Some(ts) = u.get_timestamp() {
                ts.to_unix().0
            } else {
                0 // v4 UUIDs have no timestamp
            }
        }

        fn validate(uuid: &db_types::UuidType) -> bool {
            // Verify that the UUID is valid and has a known version (v4 or v7)
            let u = Uuid::from_bytes(uuid.0);
            matches!(u.get_version_num(), 7)
        }
    }

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
}
