use crate::new_types::NonceUuid;
use crate::types::HashimError;
use crate::types::JWTError;
use crate::types::NonceError;
use infrastructure::jwt::JsonWebTokenType;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use utility::dtos::Txn;
use utility::dtos::TypeOperationsError;
use utility::dtos::TypeOperationsInput;
use utility::dtos::TypeOperationsOk;
use utility::dtos::TypeOperationsResource;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) enum FromServer {
    Error(HashimError),
    PushData(MyResult),
    Resources(Vec<TypeOperationsResource>),
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
