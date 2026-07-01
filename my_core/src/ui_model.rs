use crate::prelude::*;

pub trait HashimSignal<T: Default + Clone>: Default {
    fn reset(&self) {
        self.set(T::default());
    }
    fn read(&self) -> T;
    fn set(&self, v: T);
}

// model

#[derive(Default)]
pub struct Model<At: AllClientTypes> {
    pub navigator: At::Navigator,
    pub page_root: PageRoot<At>,
    pub external_errors: At::StringVec,
}

#[derive(Default)]
pub struct PageRoot<At: AllClientTypes> {
    pub page_auth: PageAuth<At>,
    pub page_after_auth: PageAfterAuth<At>,
}

#[derive(Default)]
pub struct PageAuth<At: AllClientTypes> {
    pub auth_feature_state: AuthFeatureState<At>,
    pub page_sign_up: PageSignUp<At>,
    pub page_sign_in: PageSignIn<At>,
}

#[derive(Default)]
pub struct PageAfterAuth<At: AllClientTypes> {
    pub user_id: At::String,
    pub user_name: At::String,

    pub page_company_branch_selection: PageCompanyBranchSelection<At>,
    pub page_home: PageHome<At>,
}

#[derive(Default)]
pub struct PageCompanyBranchSelection<At: AllClientTypes> {
    pub list: At::CompanyAndBranchList,
    pub selected_company: At::OptionUuid,

    pub page_create_company: PageCreateCompany<At>,
    pub page_create_company_branch: PageCreateCompanyBranch<At>,
}

#[derive(Default)]
pub struct PageHome<At: AllClientTypes> {
    selected_branch: At::String,
}

#[derive(Default)]
pub struct AuthFeatureState<At: AllClientTypes> {
    pub user_id: At::String,
    pub user_password: At::String,
    pub is_loading: At::Bool,
}

#[derive(Default)]
pub struct PageSignIn<At: AllClientTypes> {
    pub show_dialog: At::Dialog,
    pub user_id_error: At::String,
    pub user_password_error: At::String,
}

#[derive(Default)]
pub struct PageSignUp<At: AllClientTypes> {
    pub show_dialog: At::Dialog,
    pub user_name: At::String,
    pub user_id_error: At::String,
    pub user_name_error: At::String,
}

#[derive(Default)]
pub struct PageCreateCompany<At: AllClientTypes> {
    pub company_name: At::String,
    pub currency: At::Currency,
}

#[derive(Default)]
pub struct PageCreateCompanyBranch<At: AllClientTypes> {
    pub is_loading: At::Bool,
    pub show_dialog: At::Dialog,
    pub currency: At::Currency,
    pub branch_name: At::String,
    pub location: At::Location,
}

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

// message

#[derive(Debug)]
pub enum Message {
    CloseError,

    SignIn(ui_updaters::sign_in::Msg),
    SignUp(ui_updaters::sign_up::Msg),
    CompanyAndBranchSelection(ui_updaters::company_and_branch_selection::Msg),
    CreateCompany(ui_updaters::create_company::Msg),
    CreateCompanyBranch(ui_updaters::create_company_branch::Msg),
}

// helper types

#[derive(Default, Clone, PartialEq)]
pub enum Dialog {
    #[default]
    Hide,
    Show,
    Error,
}
