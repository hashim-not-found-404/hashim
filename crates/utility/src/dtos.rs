use dyn_clone::DynClone;
use infrastructure::time::Ti;
use infrastructure::time::Time;
use serde::Deserialize;
use serde::Serialize;
use std::any::Any;
use std::fmt::Debug;
use typetag::serde;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct TxnNumber(pub u64);

impl Default for TxnNumber {
    fn default() -> Self {
        Self(Ti::now_as_unix_milliseconds())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Txn<T> {
    pub txn_number: TxnNumber,
    pub operation:  T,
}

//////////////////////////////////////////////////////////////////////
#[serde]
pub trait TraitOperationDTOInput: Any + Debug {}
pub type TypeOperationDTOInput = Box<dyn TraitOperationDTOInput>;

//////////////////////////////////////////////////////////////////////
#[serde]
pub trait TraitOperationDTOOk: Any + Debug {}
pub type TypeOperationDTOOk = Box<dyn TraitOperationDTOOk>;

//////////////////////////////////////////////////////////////////////
#[serde]
pub trait TraitOperationDTOError: Any + Debug {}
pub type TypeOperationDTOError = Box<dyn TraitOperationDTOError>;

//////////////////////////////////////////////////////////////////////
#[serde]
pub trait TraitOperationDTOResource: Any + Debug + DynClone + Send + TraitOperationDTOOk {}
pub type TypeOperationDTOResource = Box<dyn TraitOperationDTOResource>;

dyn_clone::clone_trait_object!(TraitOperationDTOResource);

//////////////////////////////////////////////////////////////////////

pub fn dyn_result<Ok: TraitOperationDTOOk, Error: TraitOperationDTOError>(
    a: Result<Ok, Error>,
) -> Result<TypeOperationDTOOk, TypeOperationDTOError> {
    match a {
        Ok(a) => {
            let a: TypeOperationDTOOk = Box::new(a);
            Ok(a)
        }
        Err(a) => {
            let a: TypeOperationDTOError = Box::new(a);
            Err(a)
        }
    }
}
