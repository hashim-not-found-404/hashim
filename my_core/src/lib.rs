pub mod cache;
pub mod cache_query_operations;
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
        front_end_model_view::{AllSignalTypes, HashimSignal},
        request_response::*,
        server_methods::WSServer,
        traits::*,
        *,
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
        sync::{Arc, Mutex, RwLock},
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

pub trait LogError {
    #[must_use = "this `Result` may be an `Err` variant, which should be handled"]
    fn log(self) -> Self;
}

impl<T, E: core::fmt::Debug> LogError for Result<T, E> {
    #[inline(always)]
    #[track_caller]
    fn log(self) -> Self {
        if let Err(err) = &self {
            let location = std::panic::Location::caller();
            eprintln!(
                "called `Result::log()` on an `Err` value {:?}\nat {}:{}:{}",
                err,
                location.file(),
                location.line(),
                location.column()
            );
        }
        self
    }
}
