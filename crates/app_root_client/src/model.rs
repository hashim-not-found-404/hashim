use crate::navigator::Navigator;
use crate::use_case_home;
use kernel::new_types::AccountUuid;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use utility::ui_effect::Model;
use utility_ui::domain::HashimSignal;
use utility_ui::my_signal::MySignal;

impl Model for TypeGlobalModel {}

#[derive(Default, Deserialize, Serialize)]
pub(crate) struct TypeGlobalModel {
    // global states
    pub(crate) navigator: MySignal<Navigator>,
    pub(crate) selected_company_uuid: MySignal<Option<CompanyUuid>>,
    pub(crate) selected_company_branch_uuid: MySignal<Option<BranchUuid>>,
    pub(crate) user_id: MySignal<String>,
    pub(crate) user_name: MySignal<Option<String>>,

    // feature state
    pub(crate) feature_state_auth: FeatureStateAuth,

    // pages
    pub(crate) page_error_handler: use_case_error_handler::ui::TypeLocalModel,
    pub(crate) page_sign_up: use_case_sign_up::ui::TypeLocalModel,
    pub(crate) page_sign_in: use_case_sign_in::ui::TypeLocalModel,
    pub(crate) page_select_default_company: use_case_select_default_company::ui::TypeLocalModel,
    pub(crate) page_home: use_case_home::ui::TypeLocalModel,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct FeatureStateAuth {
    pub(crate) user_password: MySignal<String>,
    pub(crate) is_loading: MySignal<bool>,
}

impl TypeGlobalModel {
    pub fn user_uuid(&self) -> Option<UserUuid> {
        if let Some(a) = use_case_sign_in::client::user_uuid(&self.page_sign_in) {
            return Some(a);
        }
        if let Some(a) = use_case_sign_up::client::user_uuid(&self.page_sign_up) {
            return Some(a);
        }

        None
    }

    fn list_of_companies(&self) -> Vec<(CompanyUuid, String)> {
        use_case_select_default_company::client::list_of_companies(
            &self.page_select_default_company,
        )
    }
}

impl use_case_create_account::client::GlobalModel for TypeGlobalModel {
    fn user_uuid(&self) -> Option<UserUuid> {
        self.user_uuid()
    }

    fn selected_company(&self) -> Option<CompanyUuid> {
        self.selected_company_uuid.read()
    }

    fn list_of_companies(&self) -> Vec<(CompanyUuid, String)> {
        self.list_of_companies()
    }
}

impl use_case_error_handler::client::GlobalModel for TypeGlobalModel {}

impl use_case_sign_up::client::GlobalModel for TypeGlobalModel {
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

impl use_case_sign_in::client::GlobalModel for TypeGlobalModel {
    fn user_id(&self) -> impl HashimSignal<String> {
        self.user_id.clone()
    }

    fn password(&self) -> impl HashimSignal<String> {
        self.feature_state_auth.user_password.clone()
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

    fn selected_company_uuid(&self) -> impl HashimSignal<Option<CompanyUuid>> {
        self.selected_company_uuid.clone()
    }

    fn selected_company_branch_uuid(&self) -> impl HashimSignal<Option<BranchUuid>> {
        self.selected_company_branch_uuid.clone()
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
        self.selected_company_uuid.read()
    }
}

impl crate::navigator::GlobalModel for TypeGlobalModel {
    fn navigator(&self) -> impl HashimSignal<Navigator> {
        self.navigator.clone()
    }
}

impl use_case_create_account_for_branch::client::GlobalModel for TypeGlobalModel {
    fn user_uuid(&self) -> Option<UserUuid> {
        self.user_uuid()
    }

    fn selected_company_branch(&self) -> Option<BranchUuid> {
        self.selected_company_branch_uuid.read()
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
        self.selected_company_branch_uuid.read()
    }
}

impl use_case_home::client::GlobalModel for TypeGlobalModel {}
