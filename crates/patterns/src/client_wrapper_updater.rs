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
            use #crate_name::client::Change;
            use #crate_name::client::Effect;
            use #crate_name::client::Message;
            use #crate_name::client::apply;
            use #crate_name::client::effect;
            use #crate_name::client::update;
            use utility::process_manager::ProcessId;
            use utility::ui_effect::EffectorTrait;
            use utility::ui_effect::UpdaterTrait;
            use utility::ui_effect::UiContext;
            use utility::ui_effect::ApplierTrait;
            #[derive(Debug)]
            pub(crate) struct WrapperMessage(pub(crate) Message);

            impl UpdaterTrait for WrapperMessage {
                type Mdl = TypeModel;
                fn update(
                    &self,
                    model: &Self::Mdl,
                    process_id: ProcessId,
                ) -> (
                    Vec<Box<dyn ApplierTrait<Mdl = TypeModel>>>,
                    Vec<Box<dyn EffectorTrait>>,
                ) {
                    let local_model = model.#model_field_name.clone();

                    let (changes, effects) = update(self.0.clone(), local_model.as_ref(), model);

                    let appliers: Vec<Box<dyn ApplierTrait<Mdl = TypeModel>>> = changes
                        .into_iter()
                        .map(|c| Box::new(WrapperChange(c)) as Box<dyn ApplierTrait<Mdl = TypeModel>>)
                        .collect();

                    let effectors: Vec<Box<dyn EffectorTrait>> = effects
                        .into_iter()
                        .map(|e| Box::new(WrapperEffect(e)) as Box<dyn EffectorTrait>)
                        .collect();

                    (appliers, effectors)
                }
            }
            #[derive(Debug)]
            pub(crate) struct WrapperChange(pub(crate) Change);

            impl ApplierTrait for WrapperChange {
                type Mdl = TypeModel;
                fn apply(&self, model: &Self::Mdl, process_id: ProcessId) {
                    let local_model = model.#model_field_name.clone();
                    apply(self.0.clone(), local_model.as_ref(), model);
                }
            }
            #[derive(Debug)]
            pub(crate) struct WrapperEffect(pub(crate) Effect);

            impl EffectorTrait for WrapperEffect {
                fn effect(
                    &self,
                    process_id: ProcessId,
                    context: UiContext,
                ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
                    Box::pin(effect(self.0.clone(), process_id, context))
                }
            }
        }
    }
    .into()
}
