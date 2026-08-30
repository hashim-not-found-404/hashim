use serde::Deserialize;
use serde::Serialize;
use std::ops::Deref;

#[derive(Debug, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct UuidType(pub [u8; 16]);

macro_rules! make_type {
    ($inner_type:ty, $type_name:ident) => {
        #[derive(
            Debug, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
        )]
        pub struct $type_name(pub UuidType);

        impl Deref for $type_name {
            type Target = UuidType;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl From<$inner_type> for $type_name {
            fn from(value: $inner_type) -> Self {
                Self(value)
            }
        }
    };
}

make_type!(UuidType, NonceUuid);
make_type!(UuidType, UserUuid);
make_type!(UuidType, CompanyUuid);
make_type!(UuidType, BranchUuid);
make_type!(UuidType, AccountUuid);
make_type!(UuidType, AccountForBranchUuid);
make_type!(UuidType, SharedEntryUuid);
