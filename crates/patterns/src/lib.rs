mod caster;
mod client_wrapper_cache_check;
mod client_wrapper_cache_write;
mod client_wrapper_updater;
mod server_wrapper_read;
mod server_wrapper_write;

use proc_macro::TokenStream;

// -----------------------------------------------------------------------------
// server wrappers
// -----------------------------------------------------------------------------
#[proc_macro]
pub fn make_server_wrapper_write(input: TokenStream) -> TokenStream {
    server_wrapper_write::my_macro(input)
}

#[proc_macro]
pub fn make_server_wrapper_read(input: TokenStream) -> TokenStream {
    server_wrapper_read::my_macro(input)
}

// -----------------------------------------------------------------------------
// client wrappers
// -----------------------------------------------------------------------------
#[proc_macro]
pub fn make_client_wrapper_updater(input: TokenStream) -> TokenStream {
    client_wrapper_updater::my_macro(input)
}

#[proc_macro]
pub fn make_client_wrapper_cache_check(input: TokenStream) -> TokenStream {
    client_wrapper_cache_check::my_macro(input)
}

#[proc_macro]
pub fn make_client_wrapper_cache_write(input: TokenStream) -> TokenStream {
    client_wrapper_cache_write::my_macro(input)
}

// -----------------------------------------------------------------------------
// caster cases (used inside `impl` blocks in `wire.rs`-style files)
// -----------------------------------------------------------------------------
#[proc_macro]
pub fn cast_message_to_reducer_case(input: TokenStream) -> TokenStream {
    caster::message_to_reducer(input)
}

#[proc_macro]
pub fn cast_input_to_cache_check_case(input: TokenStream) -> TokenStream {
    caster::input_to_cache_check(input)
}

#[proc_macro]
pub fn cast_ok_to_cache_write_case(input: TokenStream) -> TokenStream {
    caster::ok_to_cache_write(input)
}

#[proc_macro]
pub fn cast_input_to_client_case(input: TokenStream) -> TokenStream {
    caster::dto_input_to_client(input)
}

#[proc_macro]
pub fn cast_ok_to_client_case(input: TokenStream) -> TokenStream {
    caster::dto_ok_to_client(input)
}

#[proc_macro]
pub fn cast_error_to_client_case(input: TokenStream) -> TokenStream {
    caster::dto_error_to_client(input)
}

#[proc_macro]
pub fn cast_input_to_server_case(input: TokenStream) -> TokenStream {
    caster::input_to_server(input)
}
