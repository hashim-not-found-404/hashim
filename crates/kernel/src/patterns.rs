// i need macro to make :
// * lib file for use case
// * write the impls of the DTO in the domain
// * write the handle input for the server
// * write the wire of the handle of the server

// new type
// write the check input for the client
// write the read for the cache
// write the write for the cache
// write the read for the server
// write the write for the server
// write the method impl of the local model
// write macro for each caster method

// write the wire of the read of the cache
// write the wire of the write of the cache
// write the wire of the model

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

#[macro_export]
macro_rules! make_the_domain_impls_and_types {
    () => {
        #[typetag::serde]
        impl utility::dtos::TraitOperationDTOInput for Input {}
        #[typetag::serde]
        impl utility::dtos::TraitOperationDTOOk for Ok {}
        #[typetag::serde]
        impl utility::dtos::TraitOperationDTOError for Error {}

        pub type MyResult = Result<Ok, Error>;
    };
}
