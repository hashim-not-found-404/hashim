use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub(crate) enum Navigator {
    #[default]
    SignIn,
    SignUp,
    GetCompaniesAndBranches(GetCompaniesAndBranches),
    Home(HomeNav),
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub(crate) enum GetCompaniesAndBranches {
    None,
    CreateCompany,
    CreateCompanyBranch,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub(crate) struct HomeNav {
    pub(crate) show_menu: bool,
    pub(crate) page_to_present: Menu,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub(crate) enum Menu {
    Dashboard,
    CreateAccount,
    CreateAccountForBranch,
    CreateJournalEntry,
}
