use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use syn::parse_macro_input;

pub(crate) fn my_macro(input: TokenStream) -> TokenStream {
    let use_case_name = parse_macro_input!(input as Ident);
    let crate_name = format_ident!("use_case_{}", use_case_name);
    let mod_name = format_ident!("mcp_provider_{}", use_case_name);
    let model_field = format_ident!("page_{}", use_case_name);

    quote! {
        pub(crate) mod #mod_name {
            use crate::model::TypeGlobalModel;
            use crate::utils::COMMANDER;
            use crate::utils::with_model;
            use anyhow::Result;
            use utility::mcp::CallInput;
            use utility::mcp::TraitMcpProvider;
            use utility::mcp::McpToolDef;
            use serde_json::Value;
            use std::future::Future;
            use std::pin::Pin;
            use #crate_name::mcp::call;
            use #crate_name::mcp::tools;

            pub(crate) struct TypeMcpProvider;

            impl TraitMcpProvider for TypeMcpProvider {
                fn tools(&self) -> Vec<McpToolDef> {
                    tools()
                }

                fn call(
                    &self,
                    tool_name: &str,
                    args: Value,
                ) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>> {
                    let tool_name = tool_name.to_owned();
                    Box::pin(async move {
                        with_model(|model: &TypeGlobalModel| {
                            call(CallInput {
                                tool_name: &tool_name,
                                args,
                                local: &model.#model_field,
                                global: model,
                                commander: &COMMANDER,
                            })
                        })
                    })
                }
            }
        }
    }
    .into()
}
