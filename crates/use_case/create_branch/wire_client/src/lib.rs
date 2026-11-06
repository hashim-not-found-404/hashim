use std::any::Any;
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use use_case_create_branch_client::Effect;
use use_case_create_branch_client::GlobalChange;
pub use use_case_create_branch_client::GlobalModel;
pub use use_case_create_branch_client::Intent;
pub use use_case_create_branch_client::LocalModel;
pub use use_case_create_branch_client::Message;
use use_case_create_branch_client::effect;
use use_case_create_branch_client::global_apply;
use use_case_create_branch_client::local_apply;
use use_case_create_branch_client::update;
pub use use_case_create_branch_domain::Error;
pub use use_case_create_branch_domain::Input;
pub use use_case_create_branch_domain::Ok;
use use_case_create_branch_mcp::call;
use use_case_create_branch_mcp::tools;
use use_case_home_client::FormEffect;
use use_case_home_client::FormGlobalChange;
use use_case_home_client::FormId;
use use_case_home_client::FormLocalModel;
use use_case_home_client::FormUpdate;
use use_case_home_mcp::form_sender;
use use_case_home_wire_client::ModelAccess as HomeModelAccess;
use use_case_prelude_wire_client::*;

make_client_wrapper_cache!(create_branch);

pub trait ModelAccess {
    fn global_model(&self) -> GlobalModel;
    fn global_model_set(&mut self, v: GlobalModel);
}

pub struct WrapperForm<Mdl> {
    pub inner: LocalModel,
    _marker: PhantomData<fn() -> Mdl>,
}

impl<Mdl> WrapperForm<Mdl> {
    pub fn into_inner(self) -> LocalModel {
        self.inner
    }
    pub fn from_inner(inner: LocalModel) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }
}

impl<Mdl> Default for WrapperForm<Mdl> {
    fn default() -> Self {
        Self {
            inner: LocalModel::default(),
            _marker: PhantomData,
        }
    }
}

impl<Mdl> Clone for WrapperForm<Mdl> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: PhantomData,
        }
    }
}

impl<Mdl> fmt::Debug for WrapperForm<Mdl> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("WrapperForm").field(&self.inner).finish()
    }
}

impl<Mdl> PartialEq for WrapperForm<Mdl> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<Mdl> FormLocalModel for WrapperForm<Mdl>
where
    Mdl: ModelAccess + 'static,
{
    fn unsubscribe_message(&self) -> Option<Box<dyn MessageTrait>> {
        let msg: Box<dyn MessageTrait> = Box::new(Message::Intent(Intent::UnSubscribe));
        Some(msg)
    }

    fn clone_user_input(&self) -> Box<dyn FormLocalModel> {
        Box::new(Self::from_inner(self.inner.clone_user_input()))
    }

    fn is_submit_ok(&self) -> bool {
        self.inner.async_state.is_submit_ok()
    }

    fn is_loading(&self) -> bool {
        self.inner.async_state.is_loading()
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
        let Some(global) = global.downcast_ref::<Mdl>() else {
            return FormUpdate::empty();
        };

        let (locals, globals, effects) = update(*message, &self.inner, &global.global_model());

        for c in locals {
            self.inner = local_apply(c, self.inner.clone());
        }

        FormUpdate {
            global: globals
                .into_iter()
                .map(|c| Box::new(WrapperGlobalChange::<Mdl>::new(c)) as Box<dyn FormGlobalChange>)
                .collect(),
            effects: effects
                .into_iter()
                .map(|e| Box::new(WrapperEffect(e)) as Box<dyn FormEffect>)
                .collect(),
        }
    }

    fn render(&self, form_id: FormId, model: &dyn Any, commander: &Commander) -> Box<dyn Any> {
        let Some(model) = model.downcast_ref::<Mdl>() else {
            return Box::new(rsx! {
                div { "create_branch: wrong model type" }
            });
        };

        let company_name_error = self
            .inner
            .company_name_error(&model.global_model())
            .map(|e| format!("{e:?}"));
        let resolved_company_uuid = self
            .inner
            .resolved_company_uuid(&model.global_model())
            .map(|u| u.to_string());
        let list_of_company_name_and_uuid = self
            .inner
            .list_of_companies_to_display(&model.global_model());

        let commander_owned = commander.clone();

        Box::new(rsx! {
            use_case_create_branch_ui::Component {
                sender: move |i| form_sender(
                    commander_owned.clone(),
                    form_id,
                    Message::Intent(i),
                ),
                show_dialog: self.inner.show_dialog.clone(),
                is_loading: self.inner.async_state.clone().is_loading(),
                company_name: self.inner.company_name.clone(),
                company_name_error,
                resolved_company_uuid,
                branch_name: self.inner.branch_name.clone(),
                location: self.inner.location.clone(),
                branch_name_error: self.inner.error_branch_name().map(|e| format!("{e:?}")),
                location_error: self.inner.error_location().map(|e| format!("{e:?}")),
                list_of_company_name_and_uuid,
            }
        })
    }
}

pub struct WrapperGlobalChange<Mdl> {
    pub inner: GlobalChange,
    _marker: PhantomData<fn() -> Mdl>,
}

impl<Mdl> WrapperGlobalChange<Mdl> {
    pub fn new(inner: GlobalChange) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }
}

impl<Mdl> Clone for WrapperGlobalChange<Mdl> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: PhantomData,
        }
    }
}

impl<Mdl> fmt::Debug for WrapperGlobalChange<Mdl> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("WrapperGlobalChange")
            .field(&self.inner)
            .finish()
    }
}

impl<Mdl> FormGlobalChange for WrapperGlobalChange<Mdl>
where
    Mdl: ModelAccess + 'static,
{
    fn apply(&self, global: &mut dyn Any) {
        let Some(g) = global.downcast_mut::<Mdl>() else {
            return;
        };
        g.global_model_set(global_apply(self.inner.clone(), g.global_model()));
    }
}

#[derive(Debug, Clone)]
pub struct WrapperEffect(pub Effect);

impl FormEffect for WrapperEffect {
    fn effect(
        &self,
        replier: Replier,
        context: UiContext,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
        Box::pin(effect(self.0.clone(), replier, context))
    }
}

pub struct TypeMcpProvider;

impl<Mdl> TraitMcpProvider<Mdl> for TypeMcpProvider
where
    Mdl: Model + HomeModelAccess + ModelAccess + 'static,
{
    fn tools(&self) -> Vec<McpToolDef> {
        tools()
    }

    fn call<'a>(
        &'a self,
        tool_name: &str,
        args: Value,
        model: &'a Mdl,
        commander: &'a Commander,
        cache: &'a CacheStruct,
    ) -> Pin<Box<dyn Future<Output = Result<Value>> + Send + 'a>> {
        let tool_name = tool_name.to_owned();
        Box::pin(async move {
            let id = args
                .get("form_id")
                .and_then(Value::as_u64)
                .map(|v| v as u32)
                .context(error_to_mcp("invalid_args", "there is no form_id field"))?;

            let form_id = FormId::from(id);
            let local_model = use_case_home_wire_client::ModelAccess::local_model(model);
            let entry =
                local_model
                    .forms
                    .iter()
                    .find(|e| e.id == form_id)
                    .context(error_to_mcp(
                        "form_not_found",
                        format!("no form with id {id}"),
                    ))?;

            let form_local_model: &dyn Any = entry.form.as_ref();

            let wrapper: &WrapperForm<Mdl> = form_local_model
                .downcast_ref::<WrapperForm<Mdl>>()
                .context(error_to_mcp(
                    "wrong_form_type",
                    format!("form {id} is not a create_branch form"),
                ))?;

            call(CallInput {
                tool_name: &tool_name,
                args,
                local: &wrapper.inner,
                global: &ModelAccess::global_model(model),
                commander,
                cache,
            })
        })
    }
}
