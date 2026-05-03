// one trait have one file
// one file have one trait
// one file maybe have multiple impls
// name of the file not equal the name of the trait

use std::error::Error;

use derive_more::Display;

pub mod authentication;
pub mod encode_decode;
pub mod functions;
pub mod jwt;
pub mod random_number;
pub mod row_id;
pub mod runtime;
pub mod web_socket_adapter;

pub mod prelude {
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

    #[cfg(target_arch = "wasm32")]
    pub(crate) use std::pin::pin;

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
    pub(crate) use gloo_net::websocket::{Message, futures::WebSocket};
    #[cfg(target_arch = "wasm32")]
    pub(crate) use gloo_timers::future::TimeoutFuture;
    #[cfg(target_arch = "wasm32")]
    pub(crate) use wasm_bindgen_futures::spawn_local;
}
