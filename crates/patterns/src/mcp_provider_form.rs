use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use syn::parse_macro_input;

pub(crate) fn my_macro(input: TokenStream) -> TokenStream {
    let use_case_name = parse_macro_input!(input as Ident);
    let crate_name = format_ident!("use_case_{}", use_case_name);
    let mod_name = format_ident!("mcp_provider_{}", use_case_name);
    let form_mod = format_ident!("{}_form", use_case_name);
    let use_case_name_str = use_case_name.to_string();

    quote! {
        pub(crate) mod #mod_name {
            use crate::model::TypeGlobalModel;
            use crate::utils::COMMANDER;
            use crate::utils::with_model;
            use crate::wire::#form_mod::WrapperForm;
            use anyhow::Context;
            use anyhow::Result;
            use kernel::mcp::CallInput;
            use kernel::mcp::McpToolDef;
            use kernel::mcp::TraitMcpProvider;
            use kernel::mcp::error_to_mcp;
            use serde_json::Value;
            use std::any::Any;
            use std::future::Future;
            use std::pin::Pin;
            use #crate_name::mcp::call;
            use #crate_name::mcp::tools;
            use use_case_home::client::FormId;

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
                        let id = args
                            .get("form_id")
                            .and_then(Value::as_u64)
                            .map(|v| v as u32)
                            .context(error_to_mcp(
                                "invalid_args",
                                "there is no form_id field",
                            ))?;

                        with_model(|model: &TypeGlobalModel| -> Result<Value> {
                            let form_id = FormId::from(id);

                            let entry = model
                                .page_home
                                .forms
                                .iter()
                                .find(|e| e.id == form_id)
                                .context(error_to_mcp(
                                    "form_not_found",
                                    format!("no form with id {id}"),
                                ))?;

                            let form_local_model: &dyn Any = entry.form.as_ref();

                            let wrapper: &WrapperForm = form_local_model
                                .downcast_ref::<WrapperForm>()
                                .context(error_to_mcp(
                                    "wrong_form_type",
                                    format!(
                                        "form {id} is not a {} form",
                                        #use_case_name_str
                                    ),
                                ))?;

                            call(CallInput {
                                tool_name: &tool_name,
                                args,
                                local: &wrapper.0,
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
