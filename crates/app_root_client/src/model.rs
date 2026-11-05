use crate::use_case_navigator::client::Navigator;
use kernel::new_types::BranchUuid;
use kernel::new_types::CompanyUuid;
use kernel::new_types::UserUuid;
use utility::ui_effect::Model;

impl Model for TypeGlobalModel {}

#[derive(Clone, Default)]
pub(crate) struct TypeGlobalModel {
    pub(crate) navigator: Navigator,
    pub(crate) selected_company_uuid: Option<CompanyUuid>,
    pub(crate) selected_company_branch_uuid: Option<BranchUuid>,
    pub(crate) user_id: String,
    pub(crate) user_name: Option<String>,
    pub(crate) password: String,

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

// ============================================================================
// Navigator — internal, still trait-based.
// ============================================================================
impl crate::use_case_navigator::client::GlobalModel for TypeGlobalModel {
    fn set_navigator(&mut self, v: Navigator) {
        self.navigator = v
    }
}

// ============================================================================
// Form use cases — GlobalAccess (2 methods, read-only global access)
// ============================================================================

impl use_case_create_account_wire_client::ModelAccess for TypeGlobalModel {
    fn global_model(&self) -> use_case_create_account_wire_client::GlobalModel {
        use_case_create_account_wire_client::GlobalModel {
            user_uuid: self.user_uuid(),
            selected_company_uuid: self.selected_company_uuid.clone(),
            list_of_companies: self.list_of_companies(),
        }
    }
    fn global_model_set(&mut self, _v: use_case_create_account_wire_client::GlobalModel) {}
}

impl use_case_create_account_for_branch_wire_client::ModelAccess for TypeGlobalModel {
    fn global_model(&self) -> use_case_create_account_for_branch_wire_client::GlobalModel {
        use_case_create_account_for_branch_wire_client::GlobalModel {
            user_uuid: self.user_uuid(),
            selected_company_branch_uuid: self.selected_company_branch_uuid.clone(),
        }
    }
    fn global_model_set(
        &mut self,
        _v: use_case_create_account_for_branch_wire_client::GlobalModel,
    ) {
    }
}

impl use_case_create_branch_wire_client::ModelAccess for TypeGlobalModel {
    fn global_model(&self) -> use_case_create_branch_wire_client::GlobalModel {
        use_case_create_branch_wire_client::GlobalModel {
            user_uuid: self.user_uuid(),
            selected_company_uuid: self.selected_company_uuid.clone(),
            list_of_companies: self.list_of_companies(),
        }
    }
    fn global_model_set(&mut self, _v: use_case_create_branch_wire_client::GlobalModel) {}
}

impl use_case_create_company_wire_client::ModelAccess for TypeGlobalModel {
    fn global_model(&self) -> use_case_create_company_wire_client::GlobalModel {
        use_case_create_company_wire_client::GlobalModel {
            user_uuid: self.user_uuid(),
        }
    }
    fn global_model_set(&mut self, _v: use_case_create_company_wire_client::GlobalModel) {}
}

impl use_case_create_journal_wire_client::ModelAccess for TypeGlobalModel {
    fn global_model(&self) -> use_case_create_journal_wire_client::GlobalModel {
        use_case_create_journal_wire_client::GlobalModel {
            user_uuid: self.user_uuid(),
            selected_company_uuid: self.selected_company_uuid.clone(),
            list_of_companies: self.list_of_companies(),
        }
    }
    fn global_model_set(&mut self, _v: use_case_create_journal_wire_client::GlobalModel) {}
}

// create_journal_entry is a stub — no GlobalAccess trait yet, no impl needed.

// ============================================================================
// Page use cases — ModelAccess (4 methods, real local pass-through)
// ============================================================================

impl use_case_sign_in_wire_client::ModelAccess for TypeGlobalModel {
    fn global_model(&self) -> use_case_sign_in_wire_client::GlobalModel {
        use_case_sign_in_wire_client::GlobalModel {
            user_id: self.user_id.clone(),
            password: self.password.clone(),
        }
    }
    fn global_model_set(&mut self, v: use_case_sign_in_wire_client::GlobalModel) {
        self.user_id = v.user_id;
        self.password = v.password;
    }
    fn local_model(&self) -> use_case_sign_in_wire_client::LocalModel {
        self.page_sign_in.clone()
    }
    fn local_model_set(&mut self, v: use_case_sign_in_wire_client::LocalModel) {
        self.page_sign_in = v;
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
        self.page_sign_up = v;
    }
}

impl use_case_home_wire_client::ModelAccess for TypeGlobalModel {
    fn global_model(&self) -> use_case_home_wire_client::GlobalModel {
        use_case_home_wire_client::GlobalModel {}
    }
    fn global_model_set(&mut self, _v: use_case_home_wire_client::GlobalModel) {}
    fn local_model(&self) -> use_case_home_wire_client::LocalModel {
        self.page_home.clone()
    }
    fn local_model_set(&mut self, v: use_case_home_wire_client::LocalModel) {
        self.page_home = v;
    }
}

impl use_case_error_handler_wire_client::ModelAccess for TypeGlobalModel {
    fn global_model(&self) -> use_case_error_handler_wire_client::GlobalModel {
        use_case_error_handler_wire_client::GlobalModel {}
    }
    fn global_model_set(&mut self, _v: use_case_error_handler_wire_client::GlobalModel) {}
    fn local_model(&self) -> use_case_error_handler_wire_client::LocalModel {
        self.page_error_handler.clone()
    }
    fn local_model_set(&mut self, v: use_case_error_handler_wire_client::LocalModel) {
        self.page_error_handler = v;
    }
}

impl use_case_select_default_company_wire_client::ModelAccess for TypeGlobalModel {
    fn global_model(&self) -> use_case_select_default_company_wire_client::GlobalModel {
        use_case_select_default_company_wire_client::GlobalModel {
            user_uuid: self.user_uuid(),
            selected_company_uuid: self.selected_company_uuid.clone(),
            selected_company_branch_uuid: self.selected_company_branch_uuid.clone(),
        }
    }
    fn global_model_set(&mut self, v: use_case_select_default_company_wire_client::GlobalModel) {
        self.selected_company_uuid = v.selected_company_uuid;
        self.selected_company_branch_uuid = v.selected_company_branch_uuid;
    }
    fn local_model(&self) -> use_case_select_default_company_wire_client::LocalModel {
        self.page_select_default_company.clone()
    }
    fn local_model_set(&mut self, v: use_case_select_default_company_wire_client::LocalModel) {
        self.page_select_default_company = v;
    }
}
