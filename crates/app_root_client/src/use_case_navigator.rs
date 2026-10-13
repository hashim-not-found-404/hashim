pub(crate) mod client {
    use anyhow::Result;
    use serde::Deserialize;
    use serde::Serialize;
    use utility::cache::ProcessId;
    use utility::ui_effect::MessageTrait;
    use utility::ui_effect::UiContext;

    #[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize, Serialize)]
    pub(crate) enum Navigator {
        #[default]
        SignIn,
        SignUp,
    }

    pub(crate) trait GlobalModel {
        fn set_navigator(&mut self, v: Navigator);
    }

    #[derive(Debug, Clone)]
    pub(crate) enum GlobalChange {
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

    pub(crate) fn update(msg: Message, _: &impl GlobalModel) -> (Vec<GlobalChange>, Vec<Effect>) {
        match msg {
            Message::Intent(intent) => match intent {
                Intent::GoToSignIn => (vec![GlobalChange::Navigator(Navigator::SignIn)], vec![]),
                Intent::GoToSignUp => (vec![GlobalChange::Navigator(Navigator::SignUp)], vec![]),
            },
            Message::Observe(observe) => match observe {},
        }
    }

    pub(crate) fn global_apply(msg: GlobalChange, global_model: &mut impl GlobalModel) {
        match msg {
            GlobalChange::Navigator(i) => global_model.set_navigator(i),
        }
    }

    pub(crate) async fn effect(
        msg: Effect,
        process_id: ProcessId,
        context: UiContext,
    ) -> Result<()> {
        Ok(())
    }
}
