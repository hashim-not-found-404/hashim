use crate::prelude::*;

pub trait HashimSignal<T: Default>: Default + Clone {
    fn reset(&self) {
        self.set(T::default());
    }
    fn read(&self) -> T;
    fn set(&self, v: T);
}

pub trait AllSignalTypes: Default + Clone {
    type String: HashimSignal<String>;
    type Dialog: HashimSignal<Dialog>;
    type Uuid: HashimSignal<db_types::UuidType>;
    type OptionUuid: HashimSignal<Option<db_types::UuidType>>;
    type Bool: HashimSignal<bool>;
    type StringVec: HashimSignal<String>;
    type Currency: HashimSignal<db_types::Currency>;
    type Location: HashimSignal<db_types::Location>;
    type CompanyAndBranchList: HashimSignal<Vec<db_types::Company>>;

    type Navigator: HashimSignal<Navigator>;
}

// model

#[derive(Default, Clone)]
pub struct Model<As: AllSignalTypes> {
    pub navigator: As::Navigator,
    pub page_root: PageRoot<As>,
    pub external_errors: As::StringVec,
}

#[derive(Default, Clone)]
pub struct PageRoot<As: AllSignalTypes> {
    pub page_auth: PageAuth<As>,
    pub page_after_auth: PageAfterAuth<As>,
}

#[derive(Default, Clone)]
pub struct PageAuth<As: AllSignalTypes> {
    pub auth_feature_state: AuthFeatureState<As>,
    pub page_sign_up: PageSignUp<As>,
    pub page_sign_in: PageSignIn<As>,
}

#[derive(Default, Clone)]
pub struct PageAfterAuth<As: AllSignalTypes> {
    pub user_id: As::String,
    pub user_name: As::String,

    pub page_company_branch_selection: PageCompanyBranchSelection<As>,
    pub page_home: PageHome<As>,
}

#[derive(Default, Clone)]
pub struct PageCompanyBranchSelection<As: AllSignalTypes> {
    pub list: As::CompanyAndBranchList,
    pub selected_company: As::OptionUuid,

    pub page_create_company: PageCreateCompany<As>,
    pub page_create_company_branch: PageCreateCompanyBranch<As>,
}

#[derive(Default, Clone)]
pub struct PageHome<As: AllSignalTypes> {
    selected_branch: As::String,
}

#[derive(Default, Clone)]
pub struct AuthFeatureState<As: AllSignalTypes> {
    pub user_id: As::String,
    pub user_password: As::String,
    pub is_loading: As::Bool,
}

#[derive(Default, Clone)]
pub struct PageSignIn<As: AllSignalTypes> {
    pub show_dialog: As::Dialog,
    pub user_id_error: As::String,
    pub user_password_error: As::String,
}

#[derive(Default, Clone)]
pub struct PageSignUp<As: AllSignalTypes> {
    pub show_dialog: As::Dialog,
    pub user_name: As::String,
    pub user_id_error: As::String,
    pub user_name_error: As::String,
}

#[derive(Default, Clone)]
pub struct PageCreateCompany<As: AllSignalTypes> {
    pub company_name: As::String,
    pub currency: As::Currency,
}

#[derive(Default, Clone)]
pub struct PageCreateCompanyBranch<As: AllSignalTypes> {
    pub is_loading: As::Bool,
    pub show_dialog: As::Dialog,
    pub currency: As::Currency,
    pub branch_name: As::String,
    pub location: As::Location,
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

pub enum Message {
    CloseError,

    SignIn(ui_effect::sign_in::Msg),
    SignUp(ui_effect::sign_up::Msg),
    CompanyAndBranchSelection(ui_effect::company_and_branch_selection::Msg),
    CreateCompany(ui_effect::create_company::Msg),
    CreateCompanyBranch(ui_effect::create_company_branch::Msg),
}

// helper types

#[derive(Default, Clone, PartialEq)]
pub enum Dialog {
    #[default]
    Hide,
    Show,
    Error,
}
