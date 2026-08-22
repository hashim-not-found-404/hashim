use serde::Deserialize;
use serde::Serialize;
use std::ops::Deref;

#[derive(Debug, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct UuidType(pub [u8; 16]);

macro_rules! make_impl {
    ($Ty:ty) => {
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

#[derive(Debug, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Nonce(pub UuidType);
make_impl!(Nonce);

#[derive(Debug, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct User(pub UuidType);
make_impl!(User);

#[derive(Debug, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Company(pub UuidType);
make_impl!(Company);

#[derive(Debug, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Branch(pub UuidType);
make_impl!(Branch);

#[derive(Debug, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Account(pub UuidType);
make_impl!(Account);

#[derive(Debug, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct AccountForBranch(pub UuidType);
make_impl!(AccountForBranch);

#[derive(Debug, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct SharedEntry(pub UuidType);
make_impl!(SharedEntry);
