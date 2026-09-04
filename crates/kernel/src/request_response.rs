use crate::new_types::NonceUuid;
use crate::types::HashimError;
use crate::types::JWTError;
use crate::types::NonceError;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
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

#[typetag::serde]
pub trait OperationsInput: Debug {}
#[typetag::serde]
pub trait OperationsOk: Debug {}
#[typetag::serde]
pub trait OperationsResult: Debug {}
#[typetag::serde]
pub trait ResourceDTO: Debug {}

pub type TypeOperationsInput = Box<dyn OperationsInput>;
pub type TypeOperationsOk = Box<dyn OperationsOk>;
pub type TypeOperationsResult = Box<dyn OperationsResult>;
pub type TypeResourceDTO = Box<dyn ResourceDTO>;
