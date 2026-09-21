mod client_wrapper_cache_check;
mod client_wrapper_cache_write;
mod client_wrapper_updater;
mod server_handler_with_write;
mod server_handler_without_write;
mod server_wrapper_read;
mod server_wrapper_write;

use proc_macro::TokenStream;

#[proc_macro]
pub fn make_server_handler_with_write(input: TokenStream) -> TokenStream {
    server_handler_with_write::my_macro(input)
}

#[proc_macro]
pub fn make_server_handler_without_write(input: TokenStream) -> TokenStream {
    server_handler_without_write::my_macro(input)
}

#[proc_macro]
pub fn make_server_wrapper_write(input: TokenStream) -> TokenStream {
    server_wrapper_write::my_macro(input)
}

#[proc_macro]
pub fn make_server_wrapper_read(input: TokenStream) -> TokenStream {
    server_wrapper_read::my_macro(input)
}

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
