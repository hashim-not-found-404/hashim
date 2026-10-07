use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use utility::process_manager::ProcessId;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::Model;
use utility::ui_effect::UiContext;
use utility_ui::domain::HashimSignal;

#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize, Serialize)]
pub(crate) enum Navigator {
    #[default]
    SignIn,
    SignUp,
    GetCompaniesAndBranches(GetCompaniesAndBranches),
    // Home(HomeNav),
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub(crate) enum GetCompaniesAndBranches {
    None,
    CreateCompany,
    CreateCompanyBranch,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub(crate) struct HomeNav {
    pub(crate) show_menu: bool,
    pub(crate) page_to_present: Menu,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub(crate) enum Menu {
    Dashboard,
    CreateAccount,
    CreateAccountForBranch,
    CreateJournalEntry,
}

pub(crate) trait GlobalModel: Model + 'static {
    fn navigator(&self) -> impl HashimSignal<Navigator>;
}

#[derive(Debug, Clone)]
pub(crate) enum Change {
    Navigator(Navigator),
}

#[derive(Debug, Clone)]
pub(crate) enum Effect {}

#[derive(Debug, Clone)]
pub(crate) enum Intent {
    GoToSignIn,
    GoToSignUp,
}

#[derive(Debug, Clone)]
pub(crate) enum Observe {}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Intent(Intent),
    Observe(Observe),
}

impl MessageTrait for Message {}

pub(crate) fn reduce(
    msg: Message,
    _: ProcessId,
    global_model: &impl GlobalModel,
) -> Result<(Vec<Change>, Vec<Effect>)> {
    match msg {
        Message::Intent(intent) => match intent {
            Intent::GoToSignIn => Ok((vec![Change::Navigator(Navigator::SignIn)], vec![])),
            Intent::GoToSignUp => Ok((vec![Change::Navigator(Navigator::SignUp)], vec![])),
        },
        Message::Observe(observe) => match observe {},
    }
}

pub(crate) fn update(msg: Change, global_model: &impl GlobalModel) {
    match msg {
        Change::Navigator(i) => global_model.navigator().set(i),
    }
}

pub(crate) async fn effect(_: Effect, _: UiContext) -> Result<()> {
    Ok(())
}

pub(crate) mod navigator_reducer {
    use crate::model::TypeModel;
    use crate::navigator::Change;
    use crate::navigator::Effect;
    use crate::navigator::Message;
    use crate::navigator::effect;
    use crate::navigator::reduce;
    use crate::navigator::update;
    use anyhow::Result;
    use std::pin::Pin;
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
            let (changes, effects) = reduce(self.0.clone(), process_id, model)?;

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
        fn update(&self, model: &Self::Mdl, _: ProcessId) {
            update(self.0.clone(), model);
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
