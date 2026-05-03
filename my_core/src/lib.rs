pub mod client_cache;
pub mod db_types;
pub mod front_end_model_view;
pub mod impls;
pub mod request_response;
pub mod traits;
pub mod translations;
pub mod web_socket;

// the reasone to add this is to make all the deps are drop in and replacement
pub mod prelude {
    pub type DynamicError = Box<dyn Error>;

    pub use crate::{
        client_cache, db_types,
        front_end_model_view::{self, Signal},
        impls,
        request_response::*,
        traits::*,
        translations,
        web_socket::{self, Coding, Runtime, WebSocketOp},
    };

    // std
    pub(crate) use std::{
        collections::{HashMap, VecDeque},
        error::Error,
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
