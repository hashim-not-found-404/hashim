pub mod cache;
pub mod db_types;
pub mod front_end_model_view;
pub mod operations;
pub mod request_response;
pub mod server_methods;
pub mod traits;
pub mod translations;
pub mod web_socket;

pub mod prelude {
    pub type DynamicError = Box<dyn Error>;

    #[cfg(target_arch = "wasm32")]
    pub use dioxus_logger;

    pub use crate::mbg; // this macro for dev only
    pub use crate::{
        front_end_model_view::Signal, request_response::*, server_methods::WSServer, traits::*, *,
    };

    pub(crate) use crate::operations::*;

    // std
    pub(crate) use std::{
        collections::{HashMap, HashSet},
        error::Error,
        fmt::Display,
        future::Future,
        hash::Hash,
        marker::PhantomData,
        result::Result as StdResult,
        str::FromStr,
        sync::Arc,
        time::{Duration, SystemTime, UNIX_EPOCH},
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
