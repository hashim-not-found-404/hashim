use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use syn::parse_macro_input;

pub fn my_macro(input: TokenStream) -> TokenStream {
    let use_case_name = parse_macro_input!(input as Ident);
    let crate_name = format_ident!("use_case_{}", use_case_name);
    let mod_name = format_ident!("updater_use_case_{}", use_case_name);
    let variant_name = format_ident!("{}", use_case_name.to_string().to_upper_camel_case());

    quote! {
        mod #mod_name {
            use crate::use_case_home::client::FormVariants;
            use crate::model::TypeGlobalModel;
            use anyhow::Result;
            use std::pin::Pin;
            use #crate_name::client::Effect;
            use #crate_name::client::GlobalChange;
            use #crate_name::client::LocalChange;
            use #crate_name::client::Message;
            use #crate_name::client::effect;
            use #crate_name::client::global_apply;
            use #crate_name::client::local_apply;
            use #crate_name::client::update;
            use utility::cache::ProcessId;
            use utility::ui_effect::ApplierTrait;
            use utility::ui_effect::EffectorTrait;
            use utility::ui_effect::UiContext;
            use utility::ui_effect::UpdaterTrait;

            #[derive(Debug)]
            pub(crate) struct WrapperMessage(pub(crate) Message);

            impl UpdaterTrait for WrapperMessage {
                type Mdl = TypeGlobalModel;

                fn update(
                    &self,
                    model: &Self::Mdl,
                    process_id: ProcessId,
                ) -> (
                    Vec<Box<dyn ApplierTrait<Mdl = TypeGlobalModel>>>,
                    Vec<Box<dyn EffectorTrait>>,
                ) {
                    let Some(form) = model
                        .page_home
                        .forms
                        .iter()
                        .find(|f| f.process_id == process_id)
                    else {
                        return (vec![], vec![]);
                    };

                    let FormVariants::#variant_name(local_model) = &form.form else {
                        return (vec![], vec![]);
                    };

                    let (local_changes, global_changes, effects) =
                        update(self.0.clone(), local_model, model);

                    let mut appliers: Vec<Box<dyn ApplierTrait<Mdl = TypeGlobalModel>>> =
                        Vec::new();

                    appliers.extend(local_changes.into_iter().map(|c| {
                        Box::new(WrapperLocalChange(c))
                            as Box<dyn ApplierTrait<Mdl = TypeGlobalModel>>
                    }));

                    appliers.extend(global_changes.into_iter().map(|c| {
                        Box::new(WrapperGlobalChange(c))
                            as Box<dyn ApplierTrait<Mdl = TypeGlobalModel>>
                    }));

                    let effectors: Vec<Box<dyn EffectorTrait>> = effects
                        .into_iter()
                        .map(|e| Box::new(WrapperEffect(e)) as Box<dyn EffectorTrait>)
                        .collect();

                    (appliers, effectors)
                }
            }

            #[derive(Debug)]
            pub(crate) struct WrapperLocalChange(pub(crate) LocalChange);

            impl ApplierTrait for WrapperLocalChange {
                type Mdl = TypeGlobalModel;

                fn apply(&self, model: &mut Self::Mdl, process_id: ProcessId) {
                    let Some(form) = model
                        .page_home
                        .forms
                        .iter_mut()
                        .find(|f| f.process_id == process_id)
                    else {
                        return;
                    };

                    let FormVariants::#variant_name(local_model) = &mut form.form else {
                        return;
                    };

                    local_apply(self.0.clone(), local_model);
                }
            }

            #[derive(Debug)]
            pub(crate) struct WrapperGlobalChange(pub(crate) GlobalChange);

            impl ApplierTrait for WrapperGlobalChange {
                type Mdl = TypeGlobalModel;

                fn apply(&self, model: &mut Self::Mdl, _process_id: ProcessId) {
                    global_apply(self.0.clone(), model);
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
