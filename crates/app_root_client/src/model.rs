use crate::use_case_navigator::client::Navigator;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use utility::ui_effect::Model;

impl Model for TypeGlobalModel {}

#[derive(Clone, Default)]
pub(crate) struct TypeGlobalModel {
    // global states
    pub(crate) navigator: Navigator,
    pub(crate) selected_company_uuid: Option<CompanyUuid>,
    pub(crate) selected_company_branch_uuid: Option<BranchUuid>,
    pub(crate) user_id: String,
    pub(crate) user_name: Option<String>,
    pub(crate) password: String,

    // pages
    pub(crate) page_error_handler: use_case_error_handler_wire_client::LocalModel,
    pub(crate) page_sign_up: use_case_sign_up_wire_client::LocalModel,
    pub(crate) page_sign_in: use_case_sign_in_wire_client::LocalModel,
    pub(crate) page_select_default_company: use_case_select_default_company_wire_client::LocalModel,
    pub(crate) page_home: use_case_home_wire_client::LocalModel,
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

impl use_case_create_account_wire_client::GlobalModel for TypeGlobalModel {
    fn user_uuid(&self) -> Option<UserUuid> {
        self.user_uuid()
    }
    fn selected_company_uuid(&self) -> Option<CompanyUuid> {
        self.selected_company_uuid.clone()
    }
    fn list_of_companies(&self) -> Vec<(CompanyUuid, String)> {
        self.list_of_companies()
    }
}

impl use_case_error_handler_wire_client::GlobalModel for TypeGlobalModel {}

impl use_case_sign_in_wire_client::GlobalModel for TypeGlobalModel {
    fn user_id(&self) -> String {
        self.user_id.clone()
    }
    fn password(&self) -> String {
        self.password.clone()
    }
    fn set_user_id(&mut self, v: String) {
        self.user_id = v
    }
    fn set_password(&mut self, v: String) {
        self.password = v
    }
}

impl use_case_create_company_wire_client::GlobalModel for TypeGlobalModel {
    fn user_uuid(&self) -> Option<UserUuid> {
        self.user_uuid()
    }
}

impl use_case_select_default_company_wire_client::GlobalModel for TypeGlobalModel {
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

impl use_case_create_branch_wire_client::GlobalModel for TypeGlobalModel {
    fn list_of_companies(&self) -> Vec<(CompanyUuid, String)> {
        self.list_of_companies()
    }
    fn user_uuid(&self) -> Option<UserUuid> {
        self.user_uuid()
    }
    fn selected_company_uuid(&self) -> Option<CompanyUuid> {
        self.selected_company_uuid.clone()
    }
}

impl crate::use_case_navigator::client::GlobalModel for TypeGlobalModel {
    fn set_navigator(&mut self, v: Navigator) {
        self.navigator = v
    }
}

impl use_case_create_account_for_branch_wire_client::GlobalModel for TypeGlobalModel {
    fn user_uuid(&self) -> Option<UserUuid> {
        self.user_uuid()
    }
    fn selected_company_branch_uuid(&self) -> Option<BranchUuid> {
        self.selected_company_branch_uuid.clone()
    }
}

impl use_case_create_journal_entry_wire_client::GlobalModel for TypeGlobalModel {
    fn user_uuid(&self) -> Option<UserUuid> {
        self.user_uuid()
    }
    fn selected_company_branch_uuid(&self) -> Option<BranchUuid> {
        self.selected_company_branch_uuid.clone()
    }
}

impl use_case_home_wire_client::GlobalModel for TypeGlobalModel {}

impl use_case_create_journal_wire_client::GlobalModel for TypeGlobalModel {
    fn user_uuid(&self) -> Option<UserUuid> {
        self.user_uuid()
    }
    fn selected_company_uuid(&self) -> Option<CompanyUuid> {
        self.selected_company_uuid.clone()
    }
    fn list_of_companies(&self) -> Vec<(CompanyUuid, String)> {
        self.list_of_companies()
    }
}

impl use_case_sign_in_wire_client::HasPage for TypeGlobalModel {
    fn page(&self) -> &use_case_sign_in_wire_client::LocalModel {
        &self.page_sign_in
    }
    fn page_mut(&mut self) -> &mut use_case_sign_in_wire_client::LocalModel {
        &mut self.page_sign_in
    }
}

impl use_case_home_wire_client::HasPage for TypeGlobalModel {
    fn page(&self) -> &use_case_home_wire_client::LocalModel {
        &self.page_home
    }
    fn page_mut(&mut self) -> &mut use_case_home_wire_client::LocalModel {
        &mut self.page_home
    }
}

impl use_case_error_handler_wire_client::HasPage for TypeGlobalModel {
    fn page(&self) -> &use_case_error_handler_wire_client::LocalModel {
        &self.page_error_handler
    }
    fn page_mut(&mut self) -> &mut use_case_error_handler_wire_client::LocalModel {
        &mut self.page_error_handler
    }
}

impl use_case_select_default_company_wire_client::HasPage for TypeGlobalModel {
    fn page(&self) -> &use_case_select_default_company_wire_client::LocalModel {
        &self.page_select_default_company
    }
    fn page_mut(&mut self) -> &mut use_case_select_default_company_wire_client::LocalModel {
        &mut self.page_select_default_company
    }
}

impl use_case_sign_up_wire_client::ModelAccess for TypeGlobalModel {
    fn global_model(&self) -> use_case_sign_up_wire_client::GlobalModel {
        use_case_sign_up_wire_client::GlobalModel {
            user_name: self.user_name.clone(),
            user_id: self.user_id.clone(),
            password: self.password.clone(),
        }
    }

    fn global_model_set(&mut self, v: use_case_sign_up_wire_client::GlobalModel) {
        self.user_name = v.user_name;
        self.user_id = v.user_id;
        self.password = v.password;
    }

    fn local_model(&self) -> use_case_sign_up_wire_client::LocalModel {
        self.page_sign_up.clone()
    }

    fn local_model_set(&mut self, v: use_case_sign_up_wire_client::LocalModel) {
        self.page_sign_up = v
    }
}
