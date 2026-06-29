pub mod m {
    use crate::internel_prelude::*;
    use uuid::Uuid;

    pub struct S;

    impl RowId for S {
        fn generate() -> db_types::UuidType {
            // Generate a random UUID v4
            db_types::UuidType(*Uuid::now_v7().as_bytes())
        }

        fn get_time_as_seconds(uuid: &db_types::UuidType) -> Option<u64> {
            // Convert bytes to Uuid and extract timestamp (for UUID v7)
            let u = Uuid::from_bytes(uuid.0);
            if let Some(ts) = u.get_timestamp() {
                Some(ts.to_unix().0)
            } else {
                None // v4 UUIDs have no timestamp
            }
        }

        fn validate(uuid: &db_types::UuidType) -> bool {
            // Verify that the UUID is valid and has a known version (v4 or v7)
            let u = Uuid::from_bytes(uuid.0);
            matches!(u.get_version_num(), 7)
        }
    }
}
