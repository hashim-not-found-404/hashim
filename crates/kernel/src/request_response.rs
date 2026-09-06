use crate::new_types::NonceUuid;
use crate::types::HashimError;
use crate::types::JWTError;
use crate::types::NonceError;
use dyn_clone::DynClone;
use serde::Deserialize;
use serde::Serialize;
use std::any::Any;
use std::fmt::Debug;
use typetag::serde;
use utility::jwt::JsonWebTokenType;

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
    pub(crate) operations: Vec<Txn<TypeOperationsResult>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Txn<T> {
    pub txn_number: u64,
    pub operation:  T,
}

#[serde]
pub trait OperationsInput: Debug + DynClone {}
#[serde]
pub trait OperationsOk: Debug {}
#[serde]
pub trait OperationsResult: Debug {}
#[serde]
pub trait ResourceDTO: Debug {}

pub type TypeOperationsInput = Box<dyn OperationsInput>;
pub type TypeOperationsOk = Box<dyn OperationsOk>;
pub type TypeOperationsResult = Box<dyn OperationsResult>;
pub type TypeResourceDTO = Box<dyn ResourceDTO>;

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
