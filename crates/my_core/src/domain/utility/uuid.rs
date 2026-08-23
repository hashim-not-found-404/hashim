use serde::Deserialize;
use serde::Serialize;
use std::ops::Deref;

#[derive(Debug, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct UuidType(pub [u8; 16]);

macro_rules! make_type {
    ($Ty:ident) => {
        #[derive(
            Debug, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize,
        )]
        pub struct $Ty(pub UuidType);

        impl Deref for $Ty {
            type Target = UuidType;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl From<UuidType> for $Ty {
            fn from(value: UuidType) -> Self {
                Self(value)
            }
        }
    };
}

make_type!(Nonce);
make_type!(User);
make_type!(Company);
make_type!(Branch);
make_type!(Account);
make_type!(AccountForBranch);
make_type!(SharedEntry);
