use crate::navigator::Navigator;
use accounting_engine::accounting_stuff::InFlowType;
use accounting_engine::accounting_stuff::OutFlowType;
use infrastructure::actors::MpscSender;
use kernel::new_types::AccountForBranchUuid;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use kernel::types::Currency;
use kernel::types::Location;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::Arc;
use utility::cache::CacheStruct;
use utility::process_manager::MessageToProcessManager;
use utility::ui_effect::Aborters;
use utility::ui_effect::Model;
use utility::ui_effect::UpdaterTrait;
use utility_ui::domain::Dialog;
use utility_ui::domain::HashimSignal;
use utility_ui::my_signal::MySignal;

impl Model for TypeModel {}

#[derive(Default)]
pub struct TypeModel {
    pub(crate) user_uuid: MySignal<Option<UserUuid>>,
    pub(crate) selected_company_branch: MySignal<Option<BranchUuid>>,
    pub(crate) selected_company: MySignal<Option<CompanyUuid>>,

    pub navigator: MySignal<Navigator>,

    // global states
    pub external_errors: MySignal<String>,
    pub user_id: MySignal<String>,
    pub user_name: MySignal<String>,

    // feature state
    pub feature_state_auth: FeatureStateAuth,

    // pages
    pub page_sign_up: PageSignUp,
    pub page_sign_in: PageSignIn,
    pub page_company_branch_selection: PageCompanyBranchSelection,
    pub page_create_company: PageCreateCompany,
    pub page_create_company_branch: PageCreateCompanyBranch,
    pub page_create_account: Arc<use_case_create_account::ui::TypeLocalModel>,
    pub page_create_account_for_branch: PageCreateAccountForBranch,
    pub page_create_journal_entry: PageCreateJournalEntry,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct FeatureStateAuth {
    pub user_password: MySignal<String>,
    pub is_loading: MySignal<bool>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageSignIn {
    pub show_dialog: MySignal<Dialog>,
    pub user_id_error: MySignal<Option<String>>,
    pub user_password_error: MySignal<Option<String>>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageSignUp {
    pub show_dialog: MySignal<Dialog>,
    pub user_id_error: MySignal<Option<String>>,
    pub user_name_error: MySignal<Option<String>>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageCompanyBranchSelection {
    // pub list: MySignal<CompanyAndBranchList>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageCreateCompany {
    pub company_name: MySignal<String>,
    pub currency: MySignal<Currency>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageCreateCompanyBranch {
    pub is_loading: MySignal<bool>,
    pub show_dialog: MySignal<Dialog>,
    pub currency: MySignal<Currency>,
    pub branch_name: MySignal<String>,
    pub location: MySignal<Location>,
    pub branch_name_error: MySignal<Option<String>>,
    pub location_error: MySignal<Option<String>>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageCreateAccountForBranch {
    // pub(crate) list_of_available_account: MySignal<Vec<Account>>,
    pub is_loading: MySignal<bool>,
    pub show_dialog: MySignal<Dialog>,
    // pub filtered_list: MySignal<AccountsSuggestionList>,
    pub account_name: MySignal<String>,
    pub outflow_type: MySignal<OutFlowType>,
    pub inflow_type: MySignal<InFlowType>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PageCreateJournalEntry {
    // pub(crate) list_of_available_account: MySignal<Vec<Account>>,
    // pub filtered_list:                    MySignal<AccountsSuggestionList>,
    pub is_loading: MySignal<bool>,
    pub show_dialog: MySignal<Dialog>,
    pub shared_entry_id: MySignal<String>,

    pub some_account_are_not_inferred: MySignal<bool>,
    pub error_container_is_empty: MySignal<bool>,
    pub not_all_entry_inferred: MySignal<bool>,
    // pub double_entries:                MySignal<JournalEntry>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DoubleEntry {
    pub entry_is_empty: bool,
    pub you_need_to_split_the_entry: bool,
    // pub debit_not_equal_credit: Option<DebitNotEqualCreditError>,
    pub singles: Vec<SingleEntry>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SingleEntry {
    pub user_input_account_name: String,
    pub(crate) inferred_account_id: Option<AccountForBranchUuid>,

    pub user_input_is_debit: Option<bool>,
    pub user_input_is_inflow: Option<bool>,
    pub user_input_quantity: Option<f64>,
    pub user_input_amount: Option<f64>,
    pub user_input_inflow_type: Option<InFlowType>,
    pub user_input_outflow_type: Option<OutFlowType>,

    pub inferred_is_debit: Option<bool>,
    pub inferred_is_inflow: Option<bool>,
    pub inferred_quantity: Option<f64>,
    pub inferred_amount: Option<f64>,
    pub inferred_inflow_type: Option<InFlowType>,
    pub inferred_outflow_type: Option<OutFlowType>,

    // Error flags
    pub quantity_and_amount_are_zero: bool,
    pub duplicate_account_in_entry: bool,
    pub inventory_is_empty: bool,
    pub the_amount_should_be_positive: bool,
    pub the_quantity_should_be_positive: bool,
    pub quantity_not_equal_amount: bool,
    pub quantity_not_equal_zero: bool,
    pub insufficient_quantity_in_inventory: Option<f64>,
    pub amount_mismatch: Option<f64>,
    pub insufficient_amount_in_inventory: Option<f64>,
}

impl use_case_create_account::client::GlobalModel for TypeModel {
    fn user_uuid(&self) -> impl HashimSignal<Option<UserUuid>> {
        self.user_uuid.clone()
    }

    fn selected_company(&self) -> impl HashimSignal<Option<CompanyUuid>> {
        self.selected_company.clone()
    }
}

struct WrapperMessage(use_case_create_account::client::Message);
impl UpdaterTrait for WrapperMessage {
    type Mdl = TypeModel;

    fn update(
        self: Box<Self>,
        model: Arc<Self::Mdl>,
        cache: CacheStruct,
        sender_to_process_manager: MpscSender<MessageToProcessManager>,
        aborters: Aborters,
    ) -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(async move {
            self.0
                .update_generic(
                    model.clone(),
                    model.page_create_account.clone(),
                    cache,
                    sender_to_process_manager,
                )
                .await
        })
    }
}
