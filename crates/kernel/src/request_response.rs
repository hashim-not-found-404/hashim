use crate::new_types::NonceUuid;
use crate::types::HashimError;
use crate::types::JWTError;
use crate::types::NonceError;
use infrastructure::jwt::JsonWebTokenType;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use utility::dtos::Txn;
use utility::dtos::TypeOperationDTOError;
use utility::dtos::TypeOperationDTOInput;
use utility::dtos::TypeOperationDTOOk;
use utility::dtos::TypeOperationDTOResource;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) enum FromServer {
    Error(HashimError),
    PushData(MyResult),
    Resources(Vec<TypeOperationDTOResource>),
}

pub(crate) type FromClient = Input;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Input {
    pub(crate) jwts: Vec<JsonWebTokenType>,
    pub(crate) nonce: NonceUuid,
    pub(crate) operations: Vec<Txn<TypeOperationDTOInput>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct MyResult {
    pub(crate) jwts: Vec<Result<(), JWTError>>,
    pub(crate) nonce: Result<(), NonceError>,
    pub(crate) operations: Vec<Txn<Result<TypeOperationDTOOk, TypeOperationDTOError>>>,
}
