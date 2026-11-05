use std::boxed::Box;
use std::future::Future;
use std::pin::Pin;
use std::vec::Vec;
use use_case_prelude_wire_client::*;
use use_case_select_default_company_client::Effect;
use use_case_select_default_company_client::GlobalChange;
pub use use_case_select_default_company_client::GlobalModel;
use use_case_select_default_company_client::LocalChange;
pub use use_case_select_default_company_client::LocalModel;
pub use use_case_select_default_company_client::Message;
use use_case_select_default_company_client::effect;
use use_case_select_default_company_client::global_apply;
use use_case_select_default_company_client::local_apply;
use use_case_select_default_company_client::update;
pub use use_case_select_default_company_ui::Component;

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
        model.global_model_set(global_apply(self.0.clone(), model.global_model()));
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
