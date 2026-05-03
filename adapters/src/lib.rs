// one trait have one file
// one file have one trait
// one file maybe have multiple impls
// name of the file not equal the name of the trait

pub mod authentication;
pub mod encode_decode;
pub mod functions;
pub mod jwt;
pub mod random_number;
pub mod row_id;
pub mod runtime;
pub mod web_socket_adapter;

pub mod prelude {
    pub use super::MyError;
    pub use crate::{
        authentication, encode_decode, functions, jwt, random_number, row_id, runtime,
        web_socket_adapter,
    };

    // my crates
    pub(crate) use my_core::prelude::*;

    // std
    pub(crate) use std::{
        sync::{Arc, LazyLock, Mutex},
        time::Duration,
    };

    // third party
    pub(crate) use chrono::{Duration as ChronoDuration, Utc};
    pub(crate) use derive_more::From;
    pub(crate) use futures_util::{
        SinkExt, StreamExt,
        stream::{SplitSink, SplitStream},
    };
    pub(crate) use getrandom::fill;
    pub(crate) use postcard::{from_bytes, to_allocvec};
    pub(crate) use serde::{Deserialize, Serialize};
    pub(crate) use tokio_tungstenite_wasm::{Message, WebSocketStream, connect};
    pub(crate) use uuid::Uuid;

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) use argon2::{
        Argon2, PasswordHasher, PasswordVerifier,
        password_hash::{PasswordHash, SaltString, rand_core::OsRng},
    };
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) use jsonwebtoken::{
        Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode,
    };
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) use regex::Regex;

    #[cfg(target_arch = "wasm32")]
    pub(crate) use futures::future::{Either, select};
    #[cfg(target_arch = "wasm32")]
    pub(crate) use gloo_timers::future::TimeoutFuture;
    #[cfg(target_arch = "wasm32")]
    pub(crate) use wasm_bindgen_futures::spawn_local;
}

#[derive(Debug, Clone)]
pub enum MyError {
    DecodingError,
    Closed,
    ServerError,
    Timeout,
    //
    OtherUnexpectedStatusCode(String),
    SomeInternalErrorOfTheServer,
    Decoding,
    CheckYourWifi,
    ErrorAtSendingRequest(String),
}

impl ToString for MyError {
    fn to_string(&self) -> String {
        match self {
            Self::OtherUnexpectedStatusCode(_) => {
                return String::from("other_unexpected_status_code");
            }
            Self::SomeInternalErrorOfTheServer => {
                return String::from("some_internal_error_of_the_server");
            }
            Self::Decoding => {
                return String::from("decoding");
            }
            Self::CheckYourWifi => {
                return String::from("check_your_wifi");
            }
            Self::ErrorAtSendingRequest(s) => {
                return String::from("error_at_sending_request");
            }
            MyError::DecodingError => todo!(),
            MyError::Closed => todo!(),
            MyError::ServerError => todo!(),
            MyError::Timeout => String::from("Timeout"),
        }
    }
}
