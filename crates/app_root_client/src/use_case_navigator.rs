pub(crate) mod client {
    use anyhow::Result;
    use serde::Deserialize;
    use serde::Serialize;
    use utility::cache::ProcessId;
    use utility::ui_effect::MessageTrait;
    use utility::ui_effect::UiContext;
    use utility_ui::domain::HashimSignal;

    #[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize, Serialize)]
    pub(crate) enum Navigator {
        #[default]
        SignIn,
        SignUp,
    }

    pub(crate) trait GlobalModel {
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

    pub(crate) fn update(msg: Message, _: &impl GlobalModel) -> (Vec<Change>, Vec<Effect>) {
        match msg {
            Message::Intent(intent) => match intent {
                Intent::GoToSignIn => (vec![Change::Navigator(Navigator::SignIn)], vec![]),
                Intent::GoToSignUp => (vec![Change::Navigator(Navigator::SignUp)], vec![]),
            },
            Message::Observe(observe) => match observe {},
        }
    }

    pub(crate) fn apply(msg: Change, global_model: &impl GlobalModel) {
        match msg {
            Change::Navigator(i) => global_model.navigator().set(i),
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
