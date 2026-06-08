pub mod m {
    use crate::prelude::*;
    use derive_more::From;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq, From, Hash)]
    pub struct S(Uuid);

    impl S {
        pub fn into_inner(&self) -> Uuid {
            self.0
        }
    }

    impl TryFrom<&db_types::UuidType> for S {
        type Error = ();
        fn try_from(value: &db_types::UuidType) -> Result<Self, ()> {
            match Uuid::parse_str(value.0.as_str()) {
                Ok(o) => return Ok(Self(o)),
                Err(_) => return Err(()),
            }
        }
    }

    impl ToString for S {
        fn to_string(&self) -> String {
            self.0.to_string()
        }
    }

    impl RowId for S {
        fn generate() -> Self {
            Self(Uuid::now_v7())
        }

        fn get_time_as_seconds(&self) -> u64 {
            self.0.get_timestamp().unwrap().to_unix().0
        }
    }
}
