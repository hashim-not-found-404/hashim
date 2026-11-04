use use_case_create_branch_client::GlobalModel;
use use_case_create_branch_client::LocalModel;
pub use use_case_create_branch_domain::Error;
pub use use_case_create_branch_domain::Input;
pub use use_case_create_branch_domain::Ok;
use use_case_prelude_wire_client::*;

make_client_wrapper_cache!(create_branch);

use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use use_case_create_branch_mcp::call;
use use_case_create_branch_mcp::tools;
use use_case_home_client::FormId;
use use_case_home_mcp::form_sender;
use use_case_home_wire_client::HasPage;

pub struct TypeMcpProvider;

impl<Mdl: Model + HasPage + GlobalModel> TraitMcpProvider<Mdl> for TypeMcpProvider {
    fn tools(&self) -> Vec<McpToolDef> {
        tools()
    }

    fn call<'a>(
        &'a self,
        tool_name: &str,
        args: Value,
        model: &'a Mdl,
        commander: &'a Commander,
    ) -> Pin<Box<dyn Future<Output = Result<Value>> + Send + 'a>> {
        let tool_name = tool_name.to_owned();
        Box::pin(async move {
            let id = args
                .get("form_id")
                .and_then(Value::as_u64)
                .map(|v| v as u32)
                .context(error_to_mcp("invalid_args", "there is no form_id field"))?;

            let form_id = FormId::from(id);
            let entry = model
                .page()
                .forms
                .iter()
                .find(|e| e.id == form_id)
                .context(error_to_mcp(
                    "form_not_found",
                    format!("no form with id {id}"),
                ))?;

            let form_local_model: &dyn Any = entry.form.as_ref();

            let wrapper: &WrapperForm =
                form_local_model
                    .downcast_ref::<WrapperForm>()
                    .context(error_to_mcp(
                        "wrong_form_type",
                        format!("form {id} is not a {} form", "create_branch"),
                    ))?;

            call(CallInput {
                tool_name: &tool_name,
                args,
                local: &wrapper.0,
                global: model,
                commander: &commander,
            })
        })
    }
}

#[component]
pub(crate) fn CreateBranchForm<G: GlobalModel + PartialEq + Clone + 'static>(
    commander: fn() -> Commander,
    form_id: FormId,
    lm: LocalModel,
    gm: G,
) -> Element {
    let company_name_error = lm.company_name_error(&gm).map(|e| format!("{e:?}"));
    let resolved_company_uuid = lm.resolved_company_uuid(&gm).map(|uuid| uuid.to_string());
    let list_of_company_name_and_uuid = lm.list_of_companies_to_display(&gm);

    rsx! {
        use_case_create_branch_ui::Component {
            sender: move |i| form_sender(
                commander().clone(),
                form_id,
                use_case_create_branch_client::Message::Intent(i),
            ),
            show_dialog: lm.show_dialog.clone(),
            is_loading: lm.async_state.clone().is_loading(),
            company_name: lm.company_name.clone(),
            company_name_error,
            resolved_company_uuid,
            branch_name: lm.branch_name.clone(),
            location: lm.location.clone(),
            branch_name_error: lm.error_branch_name().map(|e| format!("{e:?}")),
            location_error: lm.error_location().map(|e| format!("{e:?}")),
            list_of_company_name_and_uuid,
        }
    }
}

use serde::Deserialize;
use serde::Serialize;
use use_case_create_branch_client::Effect;
use use_case_create_branch_client::GlobalChange;
use use_case_create_branch_client::Intent;
use use_case_create_branch_client::Message;
use use_case_create_branch_client::effect;
use use_case_create_branch_client::global_apply;
use use_case_create_branch_client::local_apply;
use use_case_create_branch_client::update;
use use_case_home_client::FormEffect;
use use_case_home_client::FormGlobalChange;
use use_case_home_client::FormLocalModel;
use use_case_home_client::FormUpdate;

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub(crate) struct WrapperForm(pub(crate) LocalModel);

#[serde]
impl FormLocalModel for WrapperForm {
    fn unsubscribe_message(&self) -> Option<Box<dyn MessageTrait>> {
        let msg: Box<dyn MessageTrait> = Box::new(Message::Intent(Intent::UnSubscribe));
        Some(msg)
    }

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
        let other: &dyn Any = other;
        other.downcast_ref::<Self>().is_some_and(|o| self == o)
    }

    fn handle(&mut self, message: Box<dyn MessageTrait>, global: &dyn Any) -> FormUpdate {
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
                .map(|c| Box::new(WrapperGlobalChange(c)) as Box<dyn FormGlobalChange>)
                .collect(),
            effects: effects
                .into_iter()
                .map(|e| Box::new(WrapperEffect(e)) as Box<dyn FormEffect>)
                .collect(),
        }
    }

    fn render(&self, form_id: FormId) -> Box<dyn Any> {
        Box::new(rsx! {
            CreateBranchForm {
                form_id,lm: self.0.clone()
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
