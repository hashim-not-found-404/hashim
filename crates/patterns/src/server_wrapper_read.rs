use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use syn::parse_macro_input;

pub fn my_macro(input: TokenStream) -> TokenStream {
    let use_case_name = parse_macro_input!(input as Ident);
    let crate_name = format_ident!("use_case_{}", use_case_name);
    let mod_name = format_ident!("server_use_case_{}", use_case_name);

    quote! {
        mod #mod_name {
            use anyhow::Result;
            use database::db_client;
            use kernel::server::SideEffects;
            use kernel::server::TraitOperationServerInput;
            use std::pin::Pin;
            use #crate_name::database::DataBaseOp;
            use #crate_name::domain::Input;
            use #crate_name::server::handle_operation_generic;
            use utility::dtos::TypeOperationDTOError;
            use utility::dtos::TypeOperationDTOOk;
            use utility::dtos::dyn_result;

            pub(crate) struct Wrapper(pub(crate) Input);

            impl TraitOperationServerInput for Wrapper {
                type Cli = db_client::S;
                fn handle_operation<'a>(
                    self: Box<Self>,
                    side_effects: &'a mut SideEffects,
                    client: &'a mut Self::Cli,
                ) -> Pin<
                    Box<
                        dyn Future<Output = Result<Result<TypeOperationDTOOk, TypeOperationDTOError>>> + 'a,
                    >,
                > {
                    Box::pin(async move {
                        let a = handle_operation_generic::<Self::Cli, DataBaseOp>(
                            &self.0,
                            side_effects,
                            client,
                        )
                        .await?;
                        Result::Ok(dyn_result(a))
                    })
                }
            }
        }
    }
    .into()
}
