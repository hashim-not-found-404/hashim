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
            use #crate_name::client::effect;
            use #crate_name::client::reduce;
            use #crate_name::client::update;
            use utility::process_manager::ProcessId;
            use utility::ui_effect::EffectorTrait;
            use utility::ui_effect::ReducerTrait;
            use utility::ui_effect::UiContext;
            use utility::ui_effect::UpdaterTrait;

            #[derive(Debug)]
            pub(crate) struct WrapperMessage(pub(crate) Message);

            impl ReducerTrait for WrapperMessage {
                type Mdl = TypeModel;
                fn reduce(
                    &self,
                    model: &Self::Mdl,
                    process_id: ProcessId,
                ) -> Result<(
                    Vec<Box<dyn UpdaterTrait<Mdl = TypeModel>>>,
                    Vec<Box<dyn EffectorTrait>>,
                )> {
                    let local_model = model.#model_field_name.clone();

                    let (changes, effects) =
                        reduce(self.0.clone(), process_id, local_model.as_ref(), model)?;

                    let updaters: Vec<Box<dyn UpdaterTrait<Mdl = TypeModel>>> = changes
                        .into_iter()
                        .map(|c| Box::new(WrapperChange(c)) as Box<dyn UpdaterTrait<Mdl = TypeModel>>)
                        .collect();

                    let effectors: Vec<Box<dyn EffectorTrait>> = effects
                        .into_iter()
                        .map(|e| Box::new(WrapperEffect(e)) as Box<dyn EffectorTrait>)
                        .collect();

                    Ok((updaters, effectors))
                }
            }

            #[derive(Debug)]
            pub(crate) struct WrapperChange(pub(crate) Change);

            impl UpdaterTrait for WrapperChange {
                type Mdl = TypeModel;
                fn update(&self, model: &Self::Mdl, process_id: ProcessId) {
                    let local_model = model.#model_field_name.clone();
                    update(self.0.clone(), local_model.as_ref(), model);
                }
            }

            #[derive(Debug)]
            pub(crate) struct WrapperEffect(pub(crate) Effect);

            impl EffectorTrait for WrapperEffect {
                fn effect(&self, context: UiContext) -> Pin<Box<dyn Future<Output = Result<()>>>> {
                    Box::pin(effect(self.0.clone(), context))
                }
            }
        }
    }
    .into()
}
