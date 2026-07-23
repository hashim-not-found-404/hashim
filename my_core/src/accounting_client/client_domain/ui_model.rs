use crate::accounting_domain::utility::types;

pub trait HashimSignal<T: Default + Clone>: Default {
    fn reset(&self) {
        self.set(T::default());
    }
    fn read(&self) -> T;
    fn set(&self, v: T);
}

pub trait AllSignalTypes: 'static + Default + Clone {
    type String: HashimSignal<String>;
    type OptionString: HashimSignal<Option<String>>;
    type Dialog: HashimSignal<Dialog>;
    type Uuid: HashimSignal<types::UuidType>;
    type OptionUuid: HashimSignal<Option<types::UuidType>>;
    type Bool: HashimSignal<bool>;
    type StringVec: HashimSignal<String>;
    type Currency: HashimSignal<types::Currency>;
    type Location: HashimSignal<types::Location>;
    type CompanyAndBranchList: HashimSignal<Vec<types::Company>>;

    type Navigator: HashimSignal<Navigator>;
}

// helper types

#[derive(Default, Clone, PartialEq)]
pub enum Dialog {
    #[default]
    Hide,
    Show,
    Error,
}

// model

#[derive(Default)]
pub struct Model<As: AllSignalTypes> {
    pub navigator: As::Navigator,
    pub page_root: PageRoot<As>,
    pub external_errors: As::StringVec,
}

#[derive(Default)]
pub struct PageRoot<As: AllSignalTypes> {
    pub page_auth: PageAuth<As>,
    pub page_after_auth: PageAfterAuth<As>,
}

#[derive(Default)]
pub struct PageAuth<As: AllSignalTypes> {
    pub auth_feature_state: AuthFeatureState<As>,
    pub page_sign_up: PageSignUp<As>,
    pub page_sign_in: PageSignIn<As>,
}

#[derive(Default)]
pub struct PageAfterAuth<As: AllSignalTypes> {
    pub user_id: As::String,
    pub user_name: As::String,

    pub page_company_branch_selection: PageCompanyBranchSelection<As>,
    pub page_home: PageHome<As>,
}

#[derive(Default)]
pub struct PageCompanyBranchSelection<As: AllSignalTypes> {
    pub list: As::CompanyAndBranchList,
    pub selected_company: As::OptionUuid,

    pub page_create_company: PageCreateCompany<As>,
    pub page_create_company_branch: PageCreateCompanyBranch<As>,
}

#[derive(Default)]
pub struct PageHome<As: AllSignalTypes> {
    pub page_create_account: PageCreateAccount<As>,
}

#[derive(Default)]
pub struct AuthFeatureState<As: AllSignalTypes> {
    pub user_id: As::String,
    pub user_password: As::String,
    pub is_loading: As::Bool,
}

#[derive(Default)]
pub struct PageSignIn<As: AllSignalTypes> {
    pub show_dialog: As::Dialog,
    pub user_id_error: As::OptionString,
    pub user_password_error: As::OptionString,
}

#[derive(Default)]
pub struct PageSignUp<As: AllSignalTypes> {
    pub show_dialog: As::Dialog,
    pub user_name: As::String,
    pub user_id_error: As::OptionString,
    pub user_name_error: As::OptionString,
}

#[derive(Default)]
pub struct PageCreateCompany<As: AllSignalTypes> {
    pub company_name: As::String,
    pub currency: As::Currency,
}

#[derive(Default)]
pub struct PageCreateCompanyBranch<As: AllSignalTypes> {
    pub is_loading: As::Bool,
    pub show_dialog: As::Dialog,
    pub currency: As::Currency,
    pub branch_name: As::String,
    pub location: As::Location,
    pub branch_name_error: As::OptionString,
    pub location_error: As::OptionString,
}

#[derive(Default)]
pub struct PageCreateAccount<As: AllSignalTypes> {
    pub is_loading: As::Bool,
    pub show_dialog: As::Dialog,
    pub is_debit: As::Bool,
    pub is_permanent_account: As::Bool,
    pub account_name: As::String,
    pub notes: As::String,
    pub unit_of_measurement_of_quantity: As::String,
    pub account_name_error: As::OptionString,
}

// message

#[derive(Debug, Clone, Copy)]
pub enum UserConsent {
    WaitForServerResponse,
    DontWaitForServerResponse,
    CancelOperation,
}

#[derive(Debug)]
pub enum Message {
    CloseError,

    SignIn(SignIn),
    SignUp(SignUp),
    CompanyAndBranchSelection(CompanyAndBranchSelection),
    CreateCompany(CreateCompany),
    CreateCompanyBranch(CreateCompanyBranch),
    Home(Home),
    CreateAccount(CreateAccount),
}

#[derive(Debug)]
pub enum SignUp {
    Submit,
    Consent(UserConsent),
    UserName(String),
    UserId(String),
    Password(String),
}

#[derive(Debug)]
pub enum SignIn {
    Submit,
    Consent(UserConsent),
    UserId(String),
    Password(String),
}

#[derive(Debug)]
pub enum CompanyAndBranchSelection {
    Subscribe,
    UnSubscribe,
    ShowCreateCompany,
    ShowCreateCompanyBranch,
    SelectedCompany(types::UuidType),
    SelectedCompanyBranch(types::UuidType),
}

#[derive(Debug)]
pub enum CreateCompany {
    Submit,
    Close,
    Name(String),
    Currency(String),
}

#[derive(Debug)]
pub enum CreateCompanyBranch {
    Submit,
    Consent(UserConsent),
    Close,
    Name(String),
    Currency(String),
}

#[derive(Debug)]
pub enum Home {
    ShowDashboard,
    ShowCreateAccount,
}

#[derive(Debug)]
pub enum CreateAccount {
    Submit,
    Consent(UserConsent),
    Clean,
    IsDebit(bool),
    IsPermanentAccount(bool),
    AccountName(String),
    Notes(String),
    UnitOfMeasurementOfQuantity(String),
}

// navigator types

#[derive(Debug, Clone)]
pub enum Navigator {
    Auth(Auth),
    CompanyBranchSelection(CompanyBranchSelection),
    Home(Menu),
}

#[derive(Debug, Clone)]
pub enum Auth {
    SignIn,
    SignUp,
}

#[derive(Debug, Clone)]
pub enum CompanyBranchSelection {
    None,
    CreateCompany,
    CreateCompanyBranch,
}

#[derive(Debug, Clone)]
pub enum Menu {
    Dashboard,
    CreateAccount,
}

impl Default for Navigator {
    fn default() -> Self {
        Self::Auth(Auth::SignIn)
    }
}
