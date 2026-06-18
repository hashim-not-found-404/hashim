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
        ext_trait::*,
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

mod ext_trait {
    use crate::prelude::*;

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

    pub(crate) trait ExtendHashMap<K, V> {
        fn insert_push(&mut self, k: K, v: V);
        fn insert_append(&mut self, k: K, v: Vec<V>);
    }

    pub(crate) trait ExtendHashMap1<K1, K2, V> {
        fn nested_insert(&mut self, k1: K1, k2: K2, v: V);
    }

    pub(crate) trait MyUpSert<K, V> {
        fn upsert<F>(&mut self, k: K, f: F)
        where
            F: FnOnce(&mut V) + Clone;
    }

    impl<K: Eq + Hash, V> ExtendHashMap<K, V> for HashMap<K, Vec<V>> {
        fn insert_push(&mut self, k: K, v: V) {
            self.entry(k).or_default().push(v);
        }

        fn insert_append(&mut self, k: K, mut v: Vec<V>) {
            self.entry(k).or_default().append(&mut v);
        }
    }

    impl<K1: Eq + Hash, K2: Eq + Hash, V> ExtendHashMap1<K1, K2, V> for HashMap<K1, HashMap<K2, V>> {
        fn nested_insert(&mut self, k1: K1, k2: K2, v: V) {
            self.entry(k1).or_default().insert(k2, v);
        }
    }

    impl<K: Eq + Hash, V: Default> MyUpSert<K, V> for HashMap<K, V> {
        fn upsert<F>(&mut self, k: K, f: F)
        where
            F: FnOnce(&mut V) + Clone,
        {
            self.entry(k).and_modify(f.clone()).or_insert({
                let mut v = V::default();
                f(&mut v);
                v
            });
        }
    }
}
