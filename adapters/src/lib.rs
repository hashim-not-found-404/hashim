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
pub mod mpsc_receiver;
pub mod mpsc_sender;
pub mod random_number;
pub mod row_id;
pub mod runtime;
pub mod web_socket_adapter;

pub mod prelude {
    pub use crate::{
        actors, authentication, encode_decode, functions, jwt, mpsc_receiver, mpsc_sender,
        random_number, row_id, runtime, web_socket_adapter,
    };

    // my crates
    pub(crate) use my_core::prelude::*;
}
