// i need macro to make :
// new type
// lib file for use case
// write the check input for the client
// write the handle input for the server
// write the read for the cache
// write the write for the cache
// write the read for the server
// write the write for the server
// write the impls of the DTO in the domain
// write the wire of the handle of the server
// write the wire of the read of the cache
// write the wire of the write of the cache
// write the wire of the model
// write the method impl of the local model
// write macro for each caster method

#[macro_export]
macro_rules! make_lib_file {
    () => {
        #[cfg(feature = "cache")]
        pub mod cache;
        #[cfg(feature = "client")]
        pub mod client;
        #[cfg(feature = "database")]
        pub mod database;
        pub mod domain;
        #[cfg(feature = "server")]
        pub mod server;
        #[cfg(feature = "ui")]
        pub mod ui;
    };
}
