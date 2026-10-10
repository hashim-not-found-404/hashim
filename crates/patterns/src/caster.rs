use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use syn::parse_macro_input;

pub fn message_to_reducer(input: TokenStream) -> TokenStream {
    let use_case_name = parse_macro_input!(input as Ident);
    let crate_name = format_ident!("use_case_{}", use_case_name);
    let wrapper_mod = format_ident!("updater_use_case_{}", use_case_name);

    quote! {
        if let Some(v) = v.downcast_ref::<#crate_name::client::Message>() {
            return Ok(Box::new(#wrapper_mod::WrapperMessage(v.clone())));
        }
    }
    .into()
}

pub fn input_to_cache_check(input: TokenStream) -> TokenStream {
    let use_case_name = parse_macro_input!(input as Ident);
    let crate_name = format_ident!("use_case_{}", use_case_name);
    let wrapper_mod = format_ident!("cache_check_use_case_{}", use_case_name);

    quote! {
        if let Some(v) = v.downcast_ref::<#crate_name::domain::Input>() {
            return Ok(Box::new(#wrapper_mod::Wrapper(v.clone())));
        }
    }
    .into()
}

pub fn ok_to_cache_write(input: TokenStream) -> TokenStream {
    let use_case_name = parse_macro_input!(input as Ident);
    let crate_name = format_ident!("use_case_{}", use_case_name);
    let wrapper_mod = format_ident!("cache_write_use_case_{}", use_case_name);

    quote! {
        if let Some(v) = v.downcast_ref::<#crate_name::domain::Ok>() {
            return Ok(Box::new(#wrapper_mod::Wrapper(v.clone())));
        }
    }
    .into()
}

pub fn dto_input_to_client(input: TokenStream) -> TokenStream {
    let use_case_name = parse_macro_input!(input as Ident);
    let crate_name = format_ident!("use_case_{}", use_case_name);

    quote! {
        if let Some(v) = v.downcast_ref::<#crate_name::domain::Input>() {
            return Ok(Arc::new(v.clone()));
        }
    }
    .into()
}

pub fn dto_ok_to_client(input: TokenStream) -> TokenStream {
    let use_case_name = parse_macro_input!(input as Ident);
    let crate_name = format_ident!("use_case_{}", use_case_name);

    quote! {
        if let Some(v) = v.downcast_ref::<#crate_name::domain::Ok>() {
            return Ok(Arc::new(v.clone()));
        }
    }
    .into()
}

pub fn dto_error_to_client(input: TokenStream) -> TokenStream {
    let use_case_name = parse_macro_input!(input as Ident);
    let crate_name = format_ident!("use_case_{}", use_case_name);

    quote! {
        if let Some(v) = v.downcast_ref::<#crate_name::domain::Error>() {
            return Ok(Box::new(v.clone()));
        }
    }
    .into()
}

pub fn input_to_server(input: TokenStream) -> TokenStream {
    let use_case_name = parse_macro_input!(input as Ident);
    let crate_name = format_ident!("use_case_{}", use_case_name);
    let wrapper_mod = format_ident!("server_use_case_{}", use_case_name);

    quote! {
        if let Some(v) = v.downcast_ref::<#crate_name::domain::Input>() {
            return Ok(Box::new(#wrapper_mod::Wrapper(v.clone())));
        }
    }
    .into()
}
