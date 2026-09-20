use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub enum Navigator {
    #[default]
    SignIn,
    SignUp,
    GetCompaniesAndBranches(GetCompaniesAndBranches),
    Home(HomeNav),
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum GetCompaniesAndBranches {
    None,
    CreateCompany,
    CreateCompanyBranch,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct HomeNav {
    pub show_menu: bool,
    pub page_to_present: Menu,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum Menu {
    Dashboard,
    CreateAccount,
    CreateAccountForBranch,
    CreateJournalEntry,
}
