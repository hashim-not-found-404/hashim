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
            use #crate_name::client::Change;
            use #crate_name::client::Effect;
            use #crate_name::client::Message;
            use #crate_name::client::apply;
            use #crate_name::client::effect;
            use #crate_name::client::update;
            use utility::cache::ProcessId;
            use utility::ui_effect::ApplierTrait;
            use utility::ui_effect::EffectorTrait;
            use utility::ui_effect::UiContext;
            use utility::ui_effect::UpdaterTrait;
            use utility_ui::domain::HashimSignal;

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
                    let forms = model.page_home.forms.read();
                    let Some(form) = forms.iter().find(|f| f.process_id == process_id) else {
                        return (vec![], vec![]);
                    };
                    let FormVariants::#variant_name(local_model) = &form.form else {
                        return (vec![], vec![]);
                    };

                    let (changes, effects) = update(self.0.clone(), local_model, model);

                    let appliers: Vec<Box<dyn ApplierTrait<Mdl = TypeGlobalModel>>> = changes
                        .into_iter()
                        .map(|c| Box::new(WrapperChange(c)) as Box<dyn ApplierTrait<Mdl = TypeGlobalModel>>)
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
                type Mdl = TypeGlobalModel;
                fn apply(&self, model: &Self::Mdl, process_id: ProcessId) {
                    let forms = &model.page_home.forms.read();
                    let Some(form) = forms.iter().find(|f| f.process_id == process_id) else {
                        return;
                    };
                    let FormVariants::#variant_name(local_model) = &form.form else {
                        return;
                    };
                    apply(self.0.clone(), local_model, model);
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
