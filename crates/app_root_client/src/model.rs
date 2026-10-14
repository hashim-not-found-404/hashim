use crate::use_case_home;
use crate::use_case_navigator::client::Navigator;
use kernel::new_types::AccountUuid;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use utility::ui_effect::Model;

impl Model for TypeGlobalModel {}

#[derive(Default, Deserialize, Serialize)]
pub(crate) struct TypeGlobalModel {
    // global states
    pub(crate) navigator: Navigator,
    pub(crate) selected_company_uuid: Option<CompanyUuid>,
    pub(crate) selected_company_branch_uuid: Option<BranchUuid>,
    pub(crate) user_id: String,
    pub(crate) user_name: Option<String>,

    // feature state
    pub(crate) feature_state_auth: FeatureStateAuth,

    // pages
    pub(crate) page_error_handler: use_case_error_handler::client::LocalModel,
    pub(crate) page_sign_up: use_case_sign_up::client::LocalModel,
    pub(crate) page_sign_in: use_case_sign_in::client::LocalModel,
    pub(crate) page_select_default_company: use_case_select_default_company::client::LocalModel,
    pub(crate) page_home: use_case_home::client::LocalModel,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct FeatureStateAuth {
    pub(crate) user_password: String,
    pub(crate) is_loading: bool,
}

impl TypeGlobalModel {
    pub fn user_uuid(&self) -> Option<UserUuid> {
        if let Some(a) = self.page_sign_in.user_uuid() {
            return Some(a);
        }
        if let Some(a) = self.page_sign_up.user_uuid() {
            return Some(a);
        }

        None
    }

    fn list_of_companies(&self) -> Vec<(CompanyUuid, String)> {
        self.page_select_default_company.list_of_companies()
    }
}

impl use_case_create_account::client::GlobalModel for TypeGlobalModel {
    fn user_uuid(&self) -> Option<UserUuid> {
        self.user_uuid()
    }

    fn selected_company(&self) -> Option<CompanyUuid> {
        self.selected_company_uuid.clone()
    }

    fn list_of_companies(&self) -> Vec<(CompanyUuid, String)> {
        self.list_of_companies()
    }
}

impl use_case_error_handler::client::GlobalModel for TypeGlobalModel {}

impl use_case_sign_up::client::GlobalModel for TypeGlobalModel {
    fn user_name(&self) -> Option<String> {
        self.user_name.clone()
    }

    fn user_id(&self) -> String {
        self.user_id.clone()
    }

    fn password(&self) -> String {
        self.feature_state_auth.user_password.clone()
    }

    fn set_user_name(&mut self, v: Option<String>) {
        self.user_name = v
    }

    fn set_user_id(&mut self, v: String) {
        self.user_id = v
    }

    fn set_password(&mut self, v: String) {
        self.feature_state_auth.user_password = v
    }
}

impl use_case_sign_in::client::GlobalModel for TypeGlobalModel {
    fn user_id(&self) -> String {
        self.user_id.clone()
    }

    fn password(&self) -> String {
        self.feature_state_auth.user_password.clone()
    }

    fn set_user_id(&mut self, v: String) {
        self.user_id = v
    }

    fn set_password(&mut self, v: String) {
        self.feature_state_auth.user_password = v
    }
}

impl use_case_create_company::client::GlobalModel for TypeGlobalModel {
    fn user_uuid(&self) -> Option<UserUuid> {
        self.user_uuid()
    }
}

impl use_case_select_default_company::client::GlobalModel for TypeGlobalModel {
    fn user_uuid(&self) -> Option<UserUuid> {
        self.user_uuid()
    }

    fn selected_company_uuid(&self) -> Option<CompanyUuid> {
        self.selected_company_uuid.clone()
    }

    fn selected_company_branch_uuid(&self) -> Option<BranchUuid> {
        self.selected_company_branch_uuid.clone()
    }

    fn set_selected_company_uuid(&mut self, v: Option<CompanyUuid>) {
        self.selected_company_uuid = v
    }

    fn set_selected_company_branch_uuid(&mut self, v: Option<BranchUuid>) {
        self.selected_company_branch_uuid = v
    }
}

impl use_case_create_branch::client::GlobalModel for TypeGlobalModel {
    fn list_of_companies(&self) -> Vec<(CompanyUuid, String)> {
        self.list_of_companies()
    }

    fn user_uuid(&self) -> Option<UserUuid> {
        self.user_uuid()
    }

    fn selected_company(&self) -> Option<CompanyUuid> {
        self.selected_company_uuid.clone()
    }
}

impl crate::use_case_navigator::client::GlobalModel for TypeGlobalModel {
    fn set_navigator(&mut self, v: Navigator) {
        self.navigator = v
    }
}

impl use_case_create_account_for_branch::client::GlobalModel for TypeGlobalModel {
    fn user_uuid(&self) -> Option<UserUuid> {
        self.user_uuid()
    }

    fn selected_company_branch(&self) -> Option<BranchUuid> {
        self.selected_company_branch_uuid.clone()
    }

    fn list_of_accounts(&self) -> Vec<(AccountUuid, String)> {
        Vec::new()
    }
}

impl use_case_create_journal_entry::client::GlobalModel for TypeGlobalModel {
    fn user_uuid(&self) -> Option<UserUuid> {
        self.user_uuid()
    }
    fn selected_company_branch(&self) -> Option<BranchUuid> {
        self.selected_company_branch_uuid.clone()
    }
}

impl use_case_home::client::GlobalModel for TypeGlobalModel {}
