// one trait have one file
// one file have one trait
// one file maybe have multiple impls
// name of the file not equal the name of the trait
// each one implementaion of trait inside module named `m` and each struct is named `S`

pub mod actors;
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
        actors, authentication, encode_decode, functions, jwt, random_number, row_id, runtime,
        web_socket_adapter,
    };
}

pub(crate) mod internel_prelude {
    pub use my_core::prelude::*;
}
