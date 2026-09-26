use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use syn::parse_macro_input;

pub fn my_macro(input: TokenStream) -> TokenStream {
    let use_case_name = parse_macro_input!(input as Ident);
    let crate_name = format_ident!("use_case_{}", use_case_name);
    let mod_name = format_ident!("updater_use_case_{}", use_case_name);
    let model_field_name = format_ident!("page_{}", use_case_name);

    quote! {
        mod #mod_name {
            use crate::model::TypeModel;
            use anyhow::Result;
            use std::pin::Pin;
            use #crate_name::client::Message;
            use #crate_name::client::update_generic;
            use utility::ui_effect::UiContext;
            use utility::ui_effect::UpdaterTrait;

            pub(crate) struct Wrapper(pub(crate) Message);

            impl UpdaterTrait for Wrapper {
                type Mdl = TypeModel;
                fn update(
                    self: Box<Self>,
                    context: UiContext<Self::Mdl>,
                ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
                    Box::pin(async move {
                        update_generic(self.0, context.model.#model_field_name.clone(), context).await?;
                        Ok(())
                    })
                }
            }
        }
    }
    .into()
}
