use use_case_prelude_wire_client::*;
pub use use_case_sign_up_client::GlobalModel;
pub use use_case_sign_up_client::LocalModel;
pub use use_case_sign_up_client::Message;
pub use use_case_sign_up_domain::Error;
pub use use_case_sign_up_domain::Input;
pub use use_case_sign_up_domain::Ok;
pub use use_case_sign_up_ui::Component;

make_client_wrapper_cache!(sign_up);

use std::boxed::Box;
use std::future::Future;
use std::pin::Pin;
use std::vec::Vec;
use use_case_sign_up_client::Effect;
use use_case_sign_up_client::GlobalChange;
use use_case_sign_up_client::LocalChange;
use use_case_sign_up_client::effect;
use use_case_sign_up_client::global_apply;
use use_case_sign_up_client::local_apply;
use use_case_sign_up_client::update;

pub trait HasPage: GlobalModel {
    fn page(&self) -> &LocalModel;

    fn page_mut(&mut self) -> &mut LocalModel;
}

#[derive(Debug)]
pub struct WrapperMessage(pub Message);

impl<Mdl: Model + HasPage> UpdaterTrait<Mdl> for WrapperMessage {
    fn update(
        &self,
        model: &Mdl,
    ) -> (Vec<Box<dyn ApplierTrait<Mdl>>>, Vec<Box<dyn EffectorTrait>>) {
        let page = model.page().clone();
        let (local_changes, global_changes, effects) = update(self.0.clone(), &page, model);
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

impl<Mdl: Model + HasPage> ApplierTrait<Mdl> for WrapperLocalChange {
    fn apply(&self, model: &mut Mdl) {
        local_apply(self.0.clone(), model.page_mut());
    }
}

#[derive(Debug)]
pub struct WrapperGlobalChange(pub GlobalChange);

impl<Mdl: Model + HasPage> ApplierTrait<Mdl> for WrapperGlobalChange {
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
