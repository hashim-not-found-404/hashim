use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use syn::parse_macro_input;

pub fn my_macro(input: TokenStream) -> TokenStream {
    let use_case_name = parse_macro_input!(input as Ident);
    let crate_name = format_ident!("use_case_{}", use_case_name);
    let camel = crate_name.to_string().to_upper_camel_case();
    let wrapper = format_ident!("Wrapper{}", camel);

    quote! {
        pub(crate) struct #wrapper(pub(crate) #crate_name::domain::Input);

        impl kernel::server::TraitOperationServerInput for #wrapper {
            type Cli = database::db_client::S;

            fn handle_operation<'a>(
                self: Box<Self>,
                side_effects: &'a mut kernel::server::SideEffects,
                client: &'a mut Self::Cli,
            ) -> std::pin::Pin<Box<
                dyn Future<Output = anyhow::Result<
                    anyhow::Result<
                        utility::dtos::TypeOperationDTOOk,
                        utility::dtos::TypeOperationDTOError,
                    >,
                >> + 'a,
            >> {
                Box::pin(async move {
                    let a = #crate_name::server::handle_operation_generic::<
                        Self::Cli,
                        #crate_name::database::DataBaseOp,
                        #crate_name::database::DataBaseOp,
                    >(&self.0, side_effects, client)
                    .await?;

                    anyhow::Result::Ok(utility::dtos::dyn_result(a))
                })
            }
        }
    }
    .into()
}
