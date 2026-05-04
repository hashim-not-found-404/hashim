pub mod client;
pub mod client_cache;
pub mod db_types;
pub mod front_end_model_view;
pub mod impls;
pub mod request_response;
pub mod traits;
pub mod translations;
pub mod web_socket;

pub mod prelude {
    pub type DynamicError = Box<dyn Error>;

    pub use crate::mbg; // this macro for dev only
    pub use crate::{
        client, client_cache, db_types,
        front_end_model_view::{self, Signal},
        impls,
        request_response::*,
        traits::*,
        translations,
        web_socket::{self, Coding, Runtime, WebSocket, WebSocketOp},
    };

    // std
    pub(crate) use std::{
        collections::{HashMap, VecDeque},
        error::Error,
        fmt::Display,
        future::Future,
        marker::PhantomData,
        pin::Pin,
        result::Result as StdResult,
        sync::{Arc, Mutex},
        task::{Context, Poll, Waker},
        time::Duration,
    };

    // third party
    pub(crate) use serde::{Deserialize, Serialize};
}

#[macro_export]
macro_rules! mbg {
    () => {
        #[cfg(not(target_arch = "wasm32"))]
        dbg!();
        #[cfg(target_arch = "wasm32")]
        dioxus_logger::tracing::info!("");
    };
    ($($val:expr),+ $(,)?) => {
        #[cfg(not(target_arch = "wasm32"))]
        ($(dbg!($val)),+,);
        #[cfg(target_arch = "wasm32")]
        ($(dioxus_logger::tracing::info!("{:?}", $val)),+,);
    };
}
