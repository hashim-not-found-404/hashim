use dyn_clone::DynClone;
use infrastructure::time::Ti;
use infrastructure::time::Time;
use serde::Deserialize;
use serde::Serialize;
use std::any::Any;
use std::fmt::Debug;
use typetag::serde;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct TxnNumber(u64);

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
pub trait OperationsInput: Any + Debug {}
pub type TypeOperationsInput = Box<dyn OperationsInput>;

//////////////////////////////////////////////////////////////////////
#[serde]
pub trait OperationsOk: Any + Debug {}
pub type TypeOperationsOk = Box<dyn OperationsOk>;

//////////////////////////////////////////////////////////////////////
#[serde]
pub trait OperationsError: Any + Debug {}
pub type TypeOperationsError = Box<dyn OperationsError>;

//////////////////////////////////////////////////////////////////////
#[serde]
pub trait ResourceDTO: Any + Debug + OperationsOk + DynClone + Send {}
pub type TypeResourceDTO = Box<dyn ResourceDTO>;

dyn_clone::clone_trait_object!(ResourceDTO);
