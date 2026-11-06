use std::boxed::Box;
use std::future::Future;
use std::pin::Pin;
use std::vec::Vec;
use use_case_home_client::Effect;
pub use use_case_home_client::FormEffect;
pub use use_case_home_client::FormEntry;
pub use use_case_home_client::FormGlobalChange;
pub use use_case_home_client::FormId;
pub use use_case_home_client::FormLocalModel;
pub use use_case_home_client::FormUpdate;
use use_case_home_client::GlobalChange;
pub use use_case_home_client::GlobalModel;
pub use use_case_home_client::Intent;
use use_case_home_client::LocalChange;
pub use use_case_home_client::LocalModel;
pub use use_case_home_client::Message;
use use_case_home_client::effect;
use use_case_home_client::global_apply;
use use_case_home_client::local_apply;
use use_case_home_client::update;
use use_case_home_mcp::call;
pub use use_case_home_mcp::form_sender;
use use_case_home_mcp::tools;
pub use use_case_home_ui::render_to_element;
use use_case_prelude_wire_client::*;

pub trait ModelAccess {
    fn global_model(&self) -> GlobalModel;
    fn global_model_set(&mut self, v: GlobalModel);
    fn local_model(&self) -> LocalModel;
    fn local_model_set(&mut self, v: LocalModel);
}

#[derive(Debug)]
pub struct WrapperMessage(pub Message);

impl<Mdl: Model + ModelAccess> UpdaterTrait<Mdl> for WrapperMessage {
    fn update(
        &self,
        model: &Mdl,
    ) -> (Vec<Box<dyn ApplierTrait<Mdl>>>, Vec<Box<dyn EffectorTrait>>) {
        let page = model.local_model().clone();
        let (local_changes, global_changes, effects) =
            update(self.0.clone(), &page, &model.global_model());
        let mut appliers: Vec<Box<dyn ApplierTrait<Mdl>>> = Vec::new();

        appliers.extend(
            local_changes
                .into_iter()
                .map(|c| Box::new(WrapperLocalChange(c)) as Box<dyn ApplierTrait<Mdl>>),
        );

        appliers.extend(
            global_changes
                .into_iter()
                .map(|c| Box::new(WrapperGlobalChange(c)) as Box<dyn ApplierTrait<Mdl>>),
        );

        let effectors: Vec<Box<dyn EffectorTrait>> = effects
            .into_iter()
            .map(|e| Box::new(WrapperEffect(e)) as Box<dyn EffectorTrait>)
            .collect();

        (appliers, effectors)
    }
}

#[derive(Debug)]
pub struct WrapperLocalChange(pub LocalChange);

impl<Mdl: Model + ModelAccess> ApplierTrait<Mdl> for WrapperLocalChange {
    fn apply(&self, model: &mut Mdl) {
        model.local_model_set(local_apply(self.0.clone(), model.local_model()));
    }
}

#[derive(Debug)]
pub struct WrapperGlobalChange(pub GlobalChange);

impl<Mdl: Model + ModelAccess> ApplierTrait<Mdl> for WrapperGlobalChange {
    fn apply(&self, model: &mut Mdl) {
        global_apply(self.0.clone(), model);
    }
}

#[derive(Debug)]
pub struct WrapperEffect(pub Effect);

impl EffectorTrait for WrapperEffect {
    fn effect(
        &self,
        replier: Replier,
        context: UiContext,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
        Box::pin(effect(self.0.clone(), replier, context))
    }
}

pub struct TypeMcpProvider;

impl<Mdl: Model + ModelAccess> TraitMcpProvider<Mdl> for TypeMcpProvider {
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
            call(CallInput {
                tool_name: &tool_name,
                args,
                local: &model.local_model(),
                global: &model.global_model(),
                commander,
                cache,
            })
        })
    }
}
