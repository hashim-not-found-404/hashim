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

    impl TryFrom<String> for S {
        type Error = ();
        fn try_from(value: String) -> Result<Self, ()> {
            match Uuid::parse_str(value.as_str()) {
                Ok(o) => return Ok(Self(o)),
                Err(_) => return Err(()),
            }
        }
    }

    impl RowId for S {}
}
