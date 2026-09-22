use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use syn::parse_macro_input;

pub fn my_macro(input: TokenStream) -> TokenStream {
    let use_case_name = parse_macro_input!(input as Ident);
    let crate_name = format_ident!("use_case_{}", use_case_name);
    let mod_name = format_ident!("updater_use_case_{}", use_case_name);

    quote! {
        mod #mod_name {
            use crate::model::TypeModel;
            use anyhow::Result;
            use infrastructure::actors::MpscSender;
            use std::pin::Pin;
            use std::sync::Arc;
            use #crate_name::client::Message;
            use #crate_name::client::update_generic;
            use utility::cache::CacheStruct;
            use utility::process_manager::MessageToProcessManager;
            use utility::ui_effect::Aborters;
            use utility::ui_effect::UpdaterTrait;

            pub(crate) struct Wrapper(pub(crate) Message);

            impl UpdaterTrait for Wrapper {
                type Mdl = TypeModel;
                fn update(
                    self: Box<Self>,
                    model: Arc<Self::Mdl>,
                    cache: CacheStruct,
                    sender_to_process_manager: MpscSender<MessageToProcessManager>,
                    aborters: Aborters,
                ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
                    Box::pin(async move {
                        update_generic(
                            self.0,
                            model.clone(),
                            model.page_create_account.clone(),
                            cache,
                            sender_to_process_manager,
                            aborters,
                        )
                        .await?;

                        Ok(())
                    })
                }
            }
        }
    }
    .into()
}
