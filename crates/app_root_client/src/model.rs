use crate::navigator::Navigator;
use accounting_engine::accounting_stuff::InFlowType;
use accounting_engine::accounting_stuff::OutFlowType;
use kernel::new_types::AccountForBranchUuid;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::types::Currency;
use kernel::types::Location;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use utility::cache::CacheStruct;
use utility::process_manager::MessageToProcessManager;
use utility::process_manager::ProcessId;
use utility::types::ReadAndSet;
use utility::ui_effect::Aborters;
use utility::ui_effect::Model;
use utility::ui_effect::UpdaterTrait;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;
use utility_ui::my_signal::MySignal;

impl Model for TypeModel {}

#[derive(Default)]
pub struct TypeModel {
    pub(crate) user_uuid:               Mutex<Option<UserUuid>>,
    pub(crate) selected_company_branch: Mutex<Option<BranchUuid>>,
    pub(crate) selected_company:        Mutex<Option<CompanyUuid>>,

    pub navigator: MySignal<Navigator>,

    // global states
    pub external_errors: MySignal<String>,
    pub user_id:         MySignal<String>,
    pub user_name:       MySignal<String>,

    // feature state
    pub feature_state_auth: FeatureStateAuth,

    // pages
    pub page_sign_up:                   PageSignUp,
    pub page_sign_in:                   PageSignIn,
    pub page_company_branch_selection:  PageCompanyBranchSelection,
    pub page_create_company:            PageCreateCompany,
    pub page_create_company_branch:     PageCreateCompanyBranch,
    pub page_create_account:            PageCreateAccount,
    pub page_create_account_for_branch: PageCreateAccountForBranch,
    pub page_create_journal_entry:      PageCreateJournalEntry,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct FeatureStateAuth {
    pub user_password: MySignal<String>,
    pub is_loading:    MySignal<bool>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageSignIn {
    pub show_dialog:         MySignal<Dialog>,
    pub user_id_error:       MySignal<Option<String>>,
    pub user_password_error: MySignal<Option<String>>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageSignUp {
    pub show_dialog:     MySignal<Dialog>,
    pub user_id_error:   MySignal<Option<String>>,
    pub user_name_error: MySignal<Option<String>>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageCompanyBranchSelection {
    // pub list: MySignal<CompanyAndBranchList>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageCreateCompany {
    pub company_name: MySignal<String>,
    pub currency:     MySignal<Currency>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageCreateCompanyBranch {
    pub is_loading:        MySignal<bool>,
    pub show_dialog:       MySignal<Dialog>,
    pub currency:          MySignal<Currency>,
    pub branch_name:       MySignal<String>,
    pub location:          MySignal<Location>,
    pub branch_name_error: MySignal<Option<String>>,
    pub location_error:    MySignal<Option<String>>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageCreateAccount {
    pub process_id: MySignal<Option<ProcessId>>,

    pub is_loading:                      MySignal<bool>,
    pub show_dialog:                     MySignal<Dialog>,
    pub is_debit:                        MySignal<bool>,
    pub is_permanent_account:            MySignal<bool>,
    pub account_name:                    MySignal<String>,
    pub notes:                           MySignal<String>,
    pub unit_of_measurement_of_quantity: MySignal<String>,
    pub account_name_error:              MySignal<Option<String>>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageCreateAccountForBranch {
    // pub(crate) list_of_available_account: Mutex<Vec<Account>>,
    pub is_loading:   MySignal<bool>,
    pub show_dialog:  MySignal<Dialog>,
    // pub filtered_list: MySignal<AccountsSuggestionList>,
    pub account_name: MySignal<String>,
    pub outflow_type: MySignal<OutFlowType>,
    pub inflow_type:  MySignal<InFlowType>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageCreateJournalEntry {
    // pub(crate) list_of_available_account: Mutex<Vec<Account>>,
    // pub filtered_list:                    MySignal<AccountsSuggestionList>,
    pub is_loading:      MySignal<bool>,
    pub show_dialog:     MySignal<Dialog>,
    pub shared_entry_id: MySignal<String>,

    pub some_account_are_not_inferred: MySignal<bool>,
    pub error_container_is_empty:      MySignal<bool>,
    pub not_all_entry_inferred:        MySignal<bool>,
    // pub double_entries:                MySignal<JournalEntry>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DoubleEntry {
    pub entry_is_empty:              bool,
    pub you_need_to_split_the_entry: bool,
    // pub debit_not_equal_credit: Option<DebitNotEqualCreditError>,
    pub singles:                     Vec<SingleEntry>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SingleEntry {
    pub user_input_account_name:    String,
    pub(crate) inferred_account_id: Option<AccountForBranchUuid>,

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

impl use_case_create_account::client::LocalModel for PageCreateAccount {
    fn process_id(&self) -> impl HashimSignal<Option<ProcessId>> {
        self.process_id.clone()
    }

    fn show_dialog(&self) -> impl HashimSignal<Dialog> {
        self.show_dialog.clone()
    }

    fn is_loading(&self) -> impl HashimSignal<bool> {
        self.is_loading.clone()
    }

    fn is_debit(&self) -> impl HashimSignal<bool> {
        self.is_debit.clone()
    }

    fn is_permanent_account(&self) -> impl HashimSignal<bool> {
        self.is_permanent_account.clone()
    }

    fn account_name(&self) -> impl HashimSignal<String> {
        self.account_name.clone()
    }

    fn notes(&self) -> impl HashimSignal<String> {
        self.notes.clone()
    }

    fn unit_of_measurement_of_quantity(&self) -> impl HashimSignal<String> {
        self.unit_of_measurement_of_quantity.clone()
    }

    fn account_name_error(&self) -> impl HashimSignal<Option<String>> {
        self.account_name_error.clone()
    }
}
