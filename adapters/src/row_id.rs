use crate::prelude::*;

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq, From)]
pub struct RowIdS(Uuid);

impl RowIdS {
    pub fn into_inner(&self) -> Uuid {
        self.0
    }
}

impl TryFrom<String> for RowIdS {
    type Error = ();
    fn try_from(value: String) -> Result<Self, ()> {
        match Uuid::parse_str(value.as_str()) {
            Ok(o) => return Ok(Self(o)),
            Err(_) => return Err(()),
        }
    }
}

impl RowId for RowIdS {
    fn generate() -> Self {
        Self(Uuid::now_v7())
    }
}
