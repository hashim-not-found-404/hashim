use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Ident;
use syn::parse_macro_input;

pub(crate) fn my_macro(input: TokenStream) -> TokenStream {
    let name = parse_macro_input!(input as Ident);
    let name_str = name.to_string();

    let module_name = format_ident!("{}_form", name_str);
    let use_case = format_ident!("use_case_{}", name_str);
    let camel = name_str.to_upper_camel_case();
    let render_fn = format_ident!("{}Form", camel);

    quote! {
        pub(crate) mod #module_name {
            use crate::app_ui::#render_fn;
            use crate::model::TypeGlobalModel;
            use anyhow::Result;
            use dioxus::prelude::*;
            use serde::Deserialize;
            use serde::Serialize;
            use std::any::Any;
            use std::pin::Pin;
            use typetag::serde;
            use #use_case::client::Effect;
            use #use_case::client::GlobalChange;
            use #use_case::client::LocalModel;
            use #use_case::client::Message;
            use #use_case::client::effect;
            use #use_case::client::global_apply;
            use #use_case::client::local_apply;
            use #use_case::client::update;
            use use_case_home::client::FormEffect;
            use use_case_home::client::FormGlobalChange;
            use use_case_home::client::FormId;
            use use_case_home::client::FormLocalModel;
            use use_case_home::client::FormUpdate;
            use utility::ui_effect::MessageTrait;
            use utility::ui_effect::Replier;
            use utility::ui_effect::UiContext;

            #[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
            pub(crate) struct WrapperForm(pub(crate) LocalModel);

            #[serde]
            impl FormLocalModel for WrapperForm {
                fn clone_user_input(&self) -> Box<dyn FormLocalModel> {
                    Box::new(WrapperForm(self.0.clone_user_input()))
                }

                fn is_submit_ok(&self) -> bool {
                    self.0.async_state.is_submit_ok()
                }

                fn is_loading(&self) -> bool {
                    self.0.async_state.is_loading()
                }

                fn dyn_eq(&self, other: &dyn FormLocalModel) -> bool {
                    let other : &dyn Any = other;
                    other
                        .downcast_ref::<Self>()
                        .is_some_and(|o| self == o)
                }

                fn handle(
                    &mut self,
                    message: Box<dyn MessageTrait>,
                    global: &dyn Any,
                ) -> FormUpdate {
                    let message: Box<dyn Any> = message;

                    let Ok(message) = message.downcast::<Message>() else {
                        return FormUpdate::empty();
                    };

                    let Some(global) = global.downcast_ref::<TypeGlobalModel>() else {
                        return FormUpdate::empty();
                    };

                    let (locals, globals, effects) = update(*message, &self.0, global);

                    for c in locals {
                        local_apply(c, &mut self.0);
                    }

                    FormUpdate {
                        global: globals
                            .into_iter()
                            .map(|c| {
                                Box::new(WrapperGlobalChange(c))
                                    as Box<dyn FormGlobalChange>
                            })
                            .collect(),
                        effects: effects
                            .into_iter()
                            .map(|e| {
                                Box::new(WrapperEffect(e)) as Box<dyn FormEffect>
                            })
                            .collect(),
                    }
                }

                fn render(&self, form_id: FormId) -> Box<dyn Any> {
                    Box::new(rsx! {
                        #render_fn {
                            form_id,
                            lm: self.0.clone()
                        }
                    })
                }
            }

            #[derive(Debug, Clone)]
            pub(crate) struct WrapperGlobalChange(pub(crate) GlobalChange);

            impl FormGlobalChange for WrapperGlobalChange {
                fn apply(&self, global: &mut dyn Any) {
                    let Some(g) = global.downcast_mut::<TypeGlobalModel>() else {
                        return;
                    };

                    global_apply(self.0.clone(), g);
                }
            }

            #[derive(Debug, Clone)]
            pub(crate) struct WrapperEffect(pub(crate) Effect);

            impl FormEffect for WrapperEffect {
                fn effect(
                    &self,
                    replier: Replier,
                    context: UiContext,
                ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
                    Box::pin(effect(self.0.clone(), replier, context))
                }
            }
        }
    }
    .into()
}
