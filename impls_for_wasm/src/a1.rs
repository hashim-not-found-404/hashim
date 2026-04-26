use derive_more::From;
use my_core::traits;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct RandomNumber;
impl traits::RandomNumber for RandomNumber {
    fn generate() -> u64 {
        let mut buf = [0u8; 8];
        getrandom::fill(&mut buf).unwrap();
        u64::from_ne_bytes(buf)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq, From)]
pub struct RowId(Uuid);

impl RowId {
    pub fn into_inner(&self) -> Uuid {
        self.0
    }
}

impl TryFrom<String> for RowId {
    type Error = ();
    fn try_from(value: String) -> Result<Self, ()> {
        match Uuid::parse_str(value.as_str()) {
            Ok(o) => return Ok(Self(o)),
            Err(_) => return Err(()),
        }
    }
}

impl traits::RowId for RowId {
    fn generate() -> Self {
        Self(Uuid::now_v7())
    }
}
