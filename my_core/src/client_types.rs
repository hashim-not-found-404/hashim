// navigator types

#[derive(Clone)]
pub enum Navigator {
    Auth(Auth),
    CompanyBranchSelection(CompanyBranchSelection),
    Home,
}

#[derive(Clone)]
pub enum Auth {
    SignIn,
    SignUp,
}

#[derive(Clone)]
pub enum CompanyBranchSelection {
    None,
    CreateCompany,
    CreateCompanyBranch,
}

impl Default for Navigator {
    fn default() -> Self {
        Self::Auth(Auth::SignIn)
    }
}

// helper types

#[derive(Default, Clone, PartialEq)]
pub enum Dialog {
    #[default]
    Hide,
    Show,
    Error,
}
