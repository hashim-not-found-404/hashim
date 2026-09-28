use crate::model::TypeModel;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use std::pin::Pin;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UiContext;
use utility::ui_effect::UpdaterTrait;
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

impl MessageTrait for Navigator {}

impl UpdaterTrait for Navigator {
    type Mdl = TypeModel;

    fn update(
        self: Box<Self>,
        context: UiContext<Self::Mdl>,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
        Box::pin(async move {
            if context.model.navigator.read() == *self {
                return Ok(());
            }
            context.model.navigator.set(*self);
            Ok(())
        })
    }
}
