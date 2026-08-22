use crate::domain::use_cases::create_journal_entry::DebitNotEqualCreditError;
use crate::domain::utility::types::Company;
use crate::domain::utility::types::Currency;
use crate::domain::utility::types::Location;
use crate::domain::utility::uuid;
use crate::domain::utility::uuid::AccountForBranch;
use crate::domain::utility::uuid::Branch;
use crate::domain::utility::uuid::User;
use crate::domain::utility::uuid::UuidType;
use crate::utility::tools::Searchable;
use accounting_engine::accounting_stuff::InFlowType;
use accounting_engine::accounting_stuff::OutFlowType;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::Mutex;

pub trait HashimSignal<T: Clone + Default>: Default {
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
    type Uuid: HashimSignal<UuidType>;
    type OptionUuid: HashimSignal<Option<UuidType>>;
    type Bool: HashimSignal<bool>;
    type StringVec: HashimSignal<String>;
    type Currency: HashimSignal<Currency>;
    type Location: HashimSignal<Location>;
    type CompanyAndBranchList: HashimSignal<Vec<Company>>;
    type OutFlowType: HashimSignal<OutFlowType>;
    type InFlowType: HashimSignal<InFlowType>;
    type AccountsSuggestionList: HashimSignal<Vec<Account>>;
    type JournalEntry: HashimSignal<Vec<DoubleEntry>>;

    type Navigator: HashimSignal<Navigator>;
}

// helper types ///////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Account {
    pub row_uuid:                        uuid::Account,
    pub is_debit:                        bool,
    pub is_permanent_account:            bool,
    pub account_name:                    String,
    pub notes:                           String,
    pub unit_of_measurement_of_quantity: String,
}

impl Searchable for Account {
    fn search_key(&self) -> String {
        self.account_name.clone()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub enum Dialog {
    #[default]
    Hide,
    Show,
    Error,
}

// model //////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Model<As: AllSignalTypes> {
    pub(crate) user_uuid:               Mutex<Option<User>>,
    pub(crate) selected_company_branch: Mutex<Option<Branch>>,
    pub(crate) selected_company:        Mutex<Option<uuid::Company>>,

    pub navigator: As::Navigator,

    // global states
    pub external_errors: As::StringVec,
    pub user_id:         As::String,
    pub user_name:       As::String,

    // feature state
    pub feature_state_auth: FeatureStateAuth<As>,

    // pages
    pub page_sign_up:                   PageSignUp<As>,
    pub page_sign_in:                   PageSignIn<As>,
    pub page_company_branch_selection:  PageCompanyBranchSelection<As>,
    pub page_create_company:            PageCreateCompany<As>,
    pub page_create_company_branch:     PageCreateCompanyBranch<As>,
    pub page_create_account:            PageCreateAccount<As>,
    pub page_create_account_for_branch: PageCreateAccountForBranch<As>,
    pub page_create_journal_entry:      PageCreateJournalEntry<As>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct FeatureStateAuth<As: AllSignalTypes> {
    pub user_password: As::String,
    pub is_loading:    As::Bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageSignIn<As: AllSignalTypes> {
    pub show_dialog:         As::Dialog,
    pub user_id_error:       As::OptionString,
    pub user_password_error: As::OptionString,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageSignUp<As: AllSignalTypes> {
    pub show_dialog:     As::Dialog,
    pub user_id_error:   As::OptionString,
    pub user_name_error: As::OptionString,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageCompanyBranchSelection<As: AllSignalTypes> {
    pub list: As::CompanyAndBranchList,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageCreateCompany<As: AllSignalTypes> {
    pub company_name: As::String,
    pub currency:     As::Currency,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageCreateCompanyBranch<As: AllSignalTypes> {
    pub is_loading:        As::Bool,
    pub show_dialog:       As::Dialog,
    pub currency:          As::Currency,
    pub branch_name:       As::String,
    pub location:          As::Location,
    pub branch_name_error: As::OptionString,
    pub location_error:    As::OptionString,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageCreateAccount<As: AllSignalTypes> {
    pub is_loading:                      As::Bool,
    pub show_dialog:                     As::Dialog,
    pub is_debit:                        As::Bool,
    pub is_permanent_account:            As::Bool,
    pub account_name:                    As::String,
    pub notes:                           As::String,
    pub unit_of_measurement_of_quantity: As::String,
    pub account_name_error:              As::OptionString,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageCreateAccountForBranch<As: AllSignalTypes> {
    pub(crate) list_of_available_account: Mutex<Vec<Account>>,

    pub is_loading:    As::Bool,
    pub show_dialog:   As::Dialog,
    pub filtered_list: As::AccountsSuggestionList,
    pub account_name:  As::String,
    pub outflow_type:  As::OutFlowType,
    pub inflow_type:   As::InFlowType,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageCreateJournalEntry<As: AllSignalTypes> {
    pub(crate) list_of_available_account: Mutex<Vec<Account>>,
    pub filtered_list:                    As::AccountsSuggestionList,

    pub is_loading:      As::Bool,
    pub show_dialog:     As::Dialog,
    pub shared_entry_id: As::String,

    pub some_account_are_not_inferred: As::Bool,
    pub error_container_is_empty:      As::Bool,
    pub not_all_entry_inferred:        As::Bool,
    pub double_entries:                As::JournalEntry,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DoubleEntry {
    pub entry_is_empty:              bool,
    pub you_need_to_split_the_entry: bool,
    pub debit_not_equal_credit:      Option<DebitNotEqualCreditError>,

    pub singles: Vec<SingleEntry>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SingleEntry {
    pub user_input_account_name:    String,
    pub(crate) inferred_account_id: Option<AccountForBranch>,

    pub user_input_is_debit:     Option<bool>,
    pub user_input_is_inflow:    Option<bool>,
    pub user_input_quantity:     Option<f64>,
    pub user_input_amount:       Option<f64>,
    pub user_input_inflow_type:  Option<InFlowType>,
    pub user_input_outflow_type: Option<OutFlowType>,

    pub inferred_is_debit:     Option<bool>,
    pub inferred_is_inflow:    Option<bool>,
    pub inferred_quantity:     Option<f64>,
    pub inferred_amount:       Option<f64>,
    pub inferred_inflow_type:  Option<InFlowType>,
    pub inferred_outflow_type: Option<OutFlowType>,

    // Error flags
    pub quantity_and_amount_are_zero:       bool,
    pub duplicate_account_in_entry:         bool,
    pub inventory_is_empty:                 bool,
    pub the_amount_should_be_positive:      bool,
    pub the_quantity_should_be_positive:    bool,
    pub quantity_not_equal_amount:          bool,
    pub quantity_not_equal_zero:            bool,
    pub insufficient_quantity_in_inventory: Option<f64>,
    pub amount_mismatch:                    Option<f64>,
    pub insufficient_amount_in_inventory:   Option<f64>,
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
    CreateAccountForBranch(CreateAccountForBranch),
    CreateJournalEntry(CreateJournalEntry),
}

#[derive(Debug)]
pub enum SignUp {
    Submit,
    Consent(UserConsent),
    UserName(String),
    UserId(String),
    Password(String),
    GoToSignIn,
}

#[derive(Debug)]
pub enum SignIn {
    Submit,
    Consent(UserConsent),
    UserId(String),
    Password(String),
    GoToSignUp,
}

#[derive(Debug)]
pub enum CompanyAndBranchSelection {
    Subscribe,
    UnSubscribe,
    ShowCreateCompany,
    ShowCreateCompanyBranch,
    SelectedCompany(uuid::Company),
    SelectedCompanyBranch(Branch),
}

#[derive(Debug)]
pub enum CreateCompany {
    Submit,
    Close,
    Name(String),
    Currency(Currency),
}

#[derive(Debug)]
pub enum CreateCompanyBranch {
    Submit,
    Consent(UserConsent),
    Close,
    Name(String),
    Currency(Currency),
}

#[derive(Debug)]
pub enum Home {
    ShowDashboard,
    ShowCreateAccount,
    ShowCreateAccountForBranch,
    ShowCreateJournalEntry,
}

#[derive(Debug)]
pub enum CreateAccount {
    Subscribe,
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
    OutflowType(OutFlowType),
    InflowType(InFlowType),
}

#[derive(Debug)]
pub enum CreateJournalEntry {
    Subscribe,
    UnSubscribe,
    SelectSuggestion {
        double_index: usize,
        single_index: usize,
        account_uuid: AccountForBranch,
    },
    Submit,
    Consent(UserConsent),
    Clean,
    AddSingleEntry {
        double_index: usize,
    },
    RemoveSingleEntry {
        double_index: usize,
        single_index: usize,
    },
    AddDoubleEntry,
    RemoveDoubleEntry {
        double_index: usize,
    },
    UpdateSingleEntry {
        double_index: usize,
        single_index: usize,
        value:        SingleEntryField,
    },
    SetSharedEntryId(String),
}

#[derive(Debug, Clone)]
pub enum SingleEntryField {
    Account(String),
    IsDebit(bool),
    IsInflow(bool),
    InflowType(InFlowType),
    OutflowType(OutFlowType),
    Amount(f64),
    Quantity(f64),
}

// navigator types ////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub enum Navigator {
    #[default]
    SignIn,
    SignUp,
    ListCompanyAndBranch(ListCompanyAndBranch),
    Home(HomeNav),
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum ListCompanyAndBranch {
    None,
    CreateCompany,
    CreateCompanyBranch,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct HomeNav {
    pub show_menu:       bool,
    pub page_to_present: Menu,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum Menu {
    Dashboard,
    CreateAccount,
    CreateAccountForBranch,
    CreateJournalEntry,
}
