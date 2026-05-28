pub mod cache_adapter;

pub mod prelude{
    pub use crate::cache_adapter;

    pub (crate) use my_core::prelude::*;
}
