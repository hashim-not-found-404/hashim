pub mod client;
pub mod client_cache;
pub mod db_types;
pub mod front_end_model_view;
pub mod request_response;
pub mod server;
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
        client, client_cache, db_types,
        front_end_model_view::{self, Signal},
        request_response::*,
        server::{self, WSServer},
        server_methods,
        traits::*,
        translations, web_socket,
    };

    // std
    pub(crate) use std::{
        collections::HashMap,
        error::Error,
        fmt::Display,
        future::Future,
        hash::Hash,
        marker::PhantomData,
        result::Result as StdResult,
        sync::{Arc, RwLock},
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
