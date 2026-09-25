use crate::navigator::Navigator;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::Arc;
use utility::ui_effect::Model;
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
    pub page_error_handler: Arc<use_case_error_handler::ui::TypeLocalModel>,
    pub user_id: MySignal<String>,
    pub user_name: MySignal<Option<String>>,

    // feature state
    pub feature_state_auth: FeatureStateAuth,

    // pages
    pub page_sign_up: Arc<use_case_sign_up::ui::TypeLocalModel>,
    // pub page_sign_in: Arc<use_case_sign_in::ui::TypeLocalModel>,
    // pub page_company_branch_selection: Arc<use_case_company_branch_selection::ui::TypeLocalModel>,
    // pub page_create_company: Arc<use_case_create_company::ui::TypeLocalModel>,
    // pub page_create_company_branch: Arc<use_case_create_company_branch::ui::TypeLocalModel>,
    pub page_create_account: Arc<use_case_create_account::ui::TypeLocalModel>,
    // pub page_create_account_for_branch: Arc<use_case_create_account_for_branch::ui::TypeLocalModel>,
    // pub page_create_journal_entry: Arc<use_case_create_journal_entry::ui::TypeLocalModel>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct FeatureStateAuth {
    pub user_password: MySignal<String>,
    pub is_loading: MySignal<bool>,
}

impl use_case_create_account::client::GlobalModel for TypeModel {
    fn user_uuid(&self) -> impl HashimSignal<Option<UserUuid>> {
        self.user_uuid.clone()
    }

    fn selected_company(&self) -> impl HashimSignal<Option<CompanyUuid>> {
        self.selected_company.clone()
    }
}

impl use_case_error_handler::client::GlobalModel for TypeModel {}

impl use_case_sign_up::client::GlobalModel for TypeModel {
    fn is_auth_loading(&self) -> impl HashimSignal<bool> {
        self.feature_state_auth.is_loading.clone()
    }

    fn user_uuid(&self) -> impl HashimSignal<Option<UserUuid>> {
        self.user_uuid.clone()
    }

    fn user_name(&self) -> impl HashimSignal<Option<String>> {
        self.user_name.clone()
    }

    fn user_id(&self) -> impl HashimSignal<String> {
        self.user_id.clone()
    }

    fn password(&self) -> impl HashimSignal<String> {
        self.feature_state_auth.user_password.clone()
    }
}
