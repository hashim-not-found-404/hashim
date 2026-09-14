use crate::new_types::NonceUuid;
use crate::server::ServerOperationsInput;
use crate::types::HashimError;
use crate::types::JWTError;
use crate::types::NonceError;
use dyn_clone::DynClone;
use infrastructure::jwt::JsonWebTokenType;
use serde::Deserialize;
use serde::Serialize;
use std::any::Any;
use std::fmt::Debug;
use typetag::serde;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) enum FromServer {
    Error(HashimError),
    PushData(MyResult),
    Resources(Vec<TypeResourceDTO>),
}

pub(crate) type FromClient = Input;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Input {
    pub(crate) jwts:       Vec<JsonWebTokenType>,
    pub(crate) nonce:      NonceUuid,
    pub(crate) operations: Vec<Txn<TypeOperationsInput>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct MyResult {
    pub(crate) jwts:       Vec<Result<(), JWTError>>,
    pub(crate) nonce:      Result<(), NonceError>,
    pub(crate) operations: Vec<Txn<Result<TypeOperationsOk, TypeOperationsError>>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Txn<T> {
    pub txn_number: u64,
    pub operation:  T,
}

//////////////////////////////////////////////////////////////////////
#[serde]
pub trait OperationsInput: Debug + DynClone + ServerOperationsInput {}
pub type TypeOperationsInput = Box<dyn OperationsInput>;

impl Clone for TypeOperationsInput {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

impl<T: OperationsInput + 'static> From<T> for TypeOperationsInput {
    fn from(input: T) -> Self {
        Box::new(input)
    }
}

//////////////////////////////////////////////////////////////////////
#[serde]
pub trait OperationsOk: Debug {}
pub type TypeOperationsOk = Box<dyn OperationsOk>;

//////////////////////////////////////////////////////////////////////
#[serde]
pub trait OperationsError: Debug {}
pub type TypeOperationsError = Box<dyn OperationsError>;

impl<T: OperationsError + 'static> From<T> for TypeOperationsError {
    fn from(input: T) -> Self {
        Box::new(input)
    }
}

//////////////////////////////////////////////////////////////////////
#[serde]
pub trait ResourceDTO: OperationsOk + Debug + DynClone + Send {}
pub type TypeResourceDTO = Box<dyn ResourceDTO>;

impl Clone for TypeResourceDTO {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}
