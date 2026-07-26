use crate::accounting_domain::utility::accounting_stuff;
use crate::accounting_domain::utility::types;
use crate::utility::tools;

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
    type OutFlowType: HashimSignal<accounting_stuff::OutFlowType>;
    type InFlowType: HashimSignal<accounting_stuff::InFlowType>;
    type AccountsSuggestionList: HashimSignal<Vec<Accounts>>;

    type Navigator: HashimSignal<Navigator>;
}

// helper types ///////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Clone, PartialEq)]
pub struct Accounts {
    pub row_uuid: types::UuidType,
    pub is_debit: bool,
    pub is_permanent_account: bool,
    pub account_name: String,
    pub notes: String,
    pub unit_of_measurement_of_quantity: String,
}

impl tools::Searchable for Accounts {
    fn search_key(&self) -> String {
        self.account_name.clone()
    }
}

#[derive(Default, Clone, PartialEq)]
pub enum Dialog {
    #[default]
    Hide,
    Show,
    Error,
}

// model //////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Model<As: AllSignalTypes> {
    pub navigator: As::Navigator,

    // global states
    pub external_errors: As::StringVec,
    pub user_id: As::String,
    pub user_name: As::String,
    pub selected_company: As::OptionUuid,

    // feature state
    pub feature_state_auth: FeatureStateAuth<As>,

    // pages
    pub page_sign_up: PageSignUp<As>,
    pub page_sign_in: PageSignIn<As>,
    pub page_company_branch_selection: PageCompanyBranchSelection<As>,
    pub page_create_company: PageCreateCompany<As>,
    pub page_create_company_branch: PageCreateCompanyBranch<As>,
    pub page_create_account: PageCreateAccount<As>,
    pub page_create_account_for_branch: PageCreateAccountForBranch<As>,
}

#[derive(Default)]
pub struct FeatureStateAuth<As: AllSignalTypes> {
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
    pub user_id_error: As::OptionString,
    pub user_name_error: As::OptionString,
}

#[derive(Default)]
pub struct PageCompanyBranchSelection<As: AllSignalTypes> {
    pub list: As::CompanyAndBranchList,
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

#[derive(Default)]
pub struct PageCreateAccountForBranch<As: AllSignalTypes> {
    pub list_of_available_account: Vec<Accounts>,

    pub is_loading: As::Bool,
    pub show_dialog: As::Dialog,
    pub filtered_list: As::AccountsSuggestionList,
    pub account_name: As::String,
    pub outflow_type: As::OutFlowType,
    pub inflow_type: As::InFlowType,
}

// message ////////////////////////////////////////////////////////////////////////////////////////

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
    Currency(types::Currency),
}

#[derive(Debug)]
pub enum CreateCompanyBranch {
    Submit,
    Consent(UserConsent),
    Close,
    Name(String),
    Currency(types::Currency),
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

#[derive(Debug)]
pub enum CreateAccountForBranch {
    Subscribe,
    UnSubscribe,
    Submit,
    Consent(UserConsent),
    Clean,
    AccountName(String),
    OutflowType(accounting_stuff::OutFlowType),
    InflowType(accounting_stuff::InFlowType),
}

// navigator types ////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub enum Navigator {
    SignIn,
    SignUp,
    ListCompanyAndBranch(ListCompanyAndBranch),
    Home(HomeNav),
}

#[derive(Debug, Clone)]
pub enum ListCompanyAndBranch {
    None,
    CreateCompany,
    CreateCompanyBranch,
}

#[derive(Debug, Clone)]
pub struct HomeNav {
    pub show_menu: bool,
    pub page_to_present: Menu,
}

#[derive(Debug, Clone)]
pub enum Menu {
    Dashboard,
    CreateAccount,
    CreateAccountForBranch,
}

impl Default for Navigator {
    fn default() -> Self {
        Self::SignIn
    }
}
