use crate::mcp_home_forms;
use crate::model::TypeGlobalModel;
use anyhow::Result;
use anyhow::bail;
use cache::cache_adapter::CacheAdapter;
use kernel::ui_construct::CastDTOToClient;
use std::any::Any;
use std::sync::Arc;
use use_case_home_wire_client::FormEffect;
use use_case_home_wire_client::FormGlobalChange;
use use_case_home_wire_client::FormId;
use use_case_home_wire_client::FormLocalModel;
use use_case_home_wire_client::FormUpdate;
use utility::cache::CastClientToCache;
use utility::cache::TraitOperationCacheInput;
use utility::cache::TraitOperationCacheOk;
use utility::cache::TypeOperationClientError;
use utility::cache::TypeOperationClientInput;
use utility::cache::TypeOperationClientOk;
use utility::dtos::TypeOperationDTOError;
use utility::dtos::TypeOperationDTOInput;
use utility::dtos::TypeOperationDTOOk;
use utility::mcp::TraitMcpProvider;
use utility::ui_effect::CastMessageToReducer;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UpdaterTrait;

pub(crate) type AccountForm = use_case_create_account_wire_client::WrapperForm<TypeGlobalModel>;
pub(crate) type AccountForBranchForm =
    use_case_create_account_for_branch_wire_client::WrapperForm<TypeGlobalModel>;
pub(crate) type BranchForm = use_case_create_branch_wire_client::WrapperForm<TypeGlobalModel>;
pub(crate) type CompanyForm = use_case_create_company_wire_client::WrapperForm<TypeGlobalModel>;
pub(crate) type JournalForm = use_case_create_journal_wire_client::WrapperForm<TypeGlobalModel>;

mod updater_use_case_navigator {
    use crate::model::TypeGlobalModel;
    use crate::use_case_navigator::client::Effect;
    use crate::use_case_navigator::client::GlobalChange;
    use crate::use_case_navigator::client::Message;
    use crate::use_case_navigator::client::effect;
    use crate::use_case_navigator::client::global_apply;
    use crate::use_case_navigator::client::update;
    use anyhow::Result;
    use std::pin::Pin;
    use utility::ui_effect::ApplierTrait;
    use utility::ui_effect::EffectorTrait;
    use utility::ui_effect::Replier;
    use utility::ui_effect::UiContext;
    use utility::ui_effect::UpdaterTrait;

    #[derive(Debug)]
    pub(crate) struct WrapperMessage(pub(crate) Message);

    impl UpdaterTrait<TypeGlobalModel> for WrapperMessage {
        fn update(
            &self,
            model: &TypeGlobalModel,
        ) -> (
            Vec<Box<dyn ApplierTrait<TypeGlobalModel>>>,
            Vec<Box<dyn EffectorTrait>>,
        ) {
            let (changes, effects) = update(self.0.clone(), model);
            let appliers: Vec<Box<dyn ApplierTrait<TypeGlobalModel>>> = changes
                .into_iter()
                .map(|c| Box::new(WrapperChange(c)) as Box<dyn ApplierTrait<TypeGlobalModel>>)
                .collect();
            let effectors: Vec<Box<dyn EffectorTrait>> = effects
                .into_iter()
                .map(|e| Box::new(WrapperEffect(e)) as Box<dyn EffectorTrait>)
                .collect();
            (appliers, effectors)
        }
    }

    #[derive(Debug)]
    pub(crate) struct WrapperChange(pub(crate) GlobalChange);

    impl ApplierTrait<TypeGlobalModel> for WrapperChange {
        fn apply(&self, model: &mut TypeGlobalModel) {
            global_apply(self.0.clone(), model);
        }
    }

    #[derive(Debug)]
    pub(crate) struct WrapperEffect(pub(crate) Effect);

    impl EffectorTrait for WrapperEffect {
        fn effect(
            &self,
            replier: Replier,
            context: UiContext,
        ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
            Box::pin(effect(self.0.clone(), replier, context))
        }
    }
}

pub(crate) fn get_all_providers() -> Vec<Box<dyn TraitMcpProvider<TypeGlobalModel>>> {
    vec![
        Box::new(mcp_home_forms::TypeMcpProvider),
        Box::new(use_case_create_branch_wire_client::TypeMcpProvider),
        Box::new(use_case_create_company_wire_client::TypeMcpProvider),
        Box::new(use_case_create_journal_wire_client::TypeMcpProvider),
        Box::new(use_case_home_wire_client::TypeMcpProvider),
    ]
}

pub(crate) struct MyCaster;

impl CastMessageToReducer for MyCaster {
    type Mdl = TypeGlobalModel;

    fn cast_message_to_reducer(
        v: Box<dyn MessageTrait>,
    ) -> Result<Box<dyn UpdaterTrait<Self::Mdl>>> {
        let v: Box<dyn Any> = v;

        if let Some(v) = v.downcast_ref::<use_case_error_handler_wire_client::Message>() {
            return Ok(Box::new(
                use_case_error_handler_wire_client::WrapperMessage(v.clone()),
            ));
        };

        if let Some(v) = v.downcast_ref::<use_case_select_default_company_wire_client::Message>() {
            return Ok(Box::new(
                use_case_select_default_company_wire_client::WrapperMessage(v.clone()),
            ));
        };

        if let Some(v) = v.downcast_ref::<crate::use_case_navigator::client::Message>() {
            return Ok(Box::new(updater_use_case_navigator::WrapperMessage(
                v.clone(),
            )));
        }

        if let Some(v) = v.downcast_ref::<use_case_home_wire_client::Message>() {
            return Ok(Box::new(use_case_home_wire_client::WrapperMessage(
                v.clone(),
            )));
        };

        if let Some(v) = v.downcast_ref::<use_case_sign_in_wire_client::Message>() {
            return Ok(Box::new(use_case_sign_in_wire_client::WrapperMessage(
                v.clone(),
            )));
        };

        if let Some(v) = v.downcast_ref::<use_case_sign_up_wire_client::Message>() {
            return Ok(Box::new(use_case_sign_up_wire_client::WrapperMessage(
                v.clone(),
            )));
        };

        bail!("downcast error in cast_message_to_reducer method")
    }
}

macro_rules! cast_input_to_cache_check_case {
    ($v:ident, $name:ident) => {
        if let Some(v) = $v.downcast_ref::<$name::Input>() {
            return Ok(::std::boxed::Box::new($name::cache_check::Wrapper(
                v.clone(),
            )));
        }
    };
}

macro_rules! cast_ok_to_cache_write_case {
    ($v:ident, $name:ident) => {
        if let Some(v) = $v.downcast_ref::<$name::Ok>() {
            return Ok(::std::boxed::Box::new($name::cache_write::Wrapper(
                v.clone(),
            )));
        }
    };
}

macro_rules! cast_input_to_client_case {
    ($v:ident, $name:ident) => {
        if let Some(v) = $v.downcast_ref::<$name::Input>() {
            return Ok(::std::sync::Arc::new(v.clone()));
        }
    };
}

macro_rules! cast_ok_to_client_case {
    ($v:ident, $name:ident) => {
        if let Some(v) = $v.downcast_ref::<$name::Ok>() {
            return Ok(::std::sync::Arc::new(v.clone()));
        }
    };
}

macro_rules! cast_error_to_client_case {
    ($v:ident, $name:ident) => {
        if let Some(v) = $v.downcast_ref::<$name::Error>() {
            return Ok(::std::boxed::Box::new(v.clone()));
        }
    };
}

impl CastClientToCache for MyCaster {
    type Cache = CacheAdapter;

    fn cast_input(
        v: TypeOperationClientInput,
    ) -> Result<Box<dyn TraitOperationCacheInput<Cache = Self::Cache>>> {
        let v: Arc<dyn Any> = v;

        cast_input_to_cache_check_case!(v, use_case_create_account_for_branch_wire_client);
        cast_input_to_cache_check_case!(v, use_case_create_account_wire_client);
        cast_input_to_cache_check_case!(v, use_case_create_branch_wire_client);
        cast_input_to_cache_check_case!(v, use_case_create_company_wire_client);
        cast_input_to_cache_check_case!(v, use_case_create_journal_entry_wire_client);
        cast_input_to_cache_check_case!(v, use_case_create_journal_wire_client);
        cast_input_to_cache_check_case!(v, use_case_get_all_accounts_wire_client);
        cast_input_to_cache_check_case!(v, use_case_get_all_journals_wire_client);
        cast_input_to_cache_check_case!(v, use_case_get_companies_and_branches_wire_client);
        cast_input_to_cache_check_case!(v, use_case_sign_in_wire_client);
        cast_input_to_cache_check_case!(v, use_case_sign_up_wire_client);

        bail!("downcast error in cast_input method")
    }

    fn cast_ok(
        v: TypeOperationClientOk,
    ) -> Result<Box<dyn TraitOperationCacheOk<Cache = Self::Cache>>> {
        let v: Arc<dyn Any> = v;

        cast_ok_to_cache_write_case!(v, use_case_create_account_for_branch_wire_client);
        cast_ok_to_cache_write_case!(v, use_case_create_account_wire_client);
        cast_ok_to_cache_write_case!(v, use_case_create_branch_wire_client);
        cast_ok_to_cache_write_case!(v, use_case_create_company_wire_client);
        cast_ok_to_cache_write_case!(v, use_case_create_journal_entry_wire_client);
        cast_ok_to_cache_write_case!(v, use_case_create_journal_wire_client);
        cast_ok_to_cache_write_case!(v, use_case_get_all_accounts_wire_client);
        cast_ok_to_cache_write_case!(v, use_case_get_all_journals_wire_client);
        cast_ok_to_cache_write_case!(v, use_case_get_companies_and_branches_wire_client);
        cast_ok_to_cache_write_case!(v, use_case_sign_in_wire_client);
        cast_ok_to_cache_write_case!(v, use_case_sign_up_wire_client);

        bail!("downcast error in cast_ok method")
    }
}

impl CastDTOToClient for MyCaster {
    fn cast_input(v: TypeOperationDTOInput) -> Result<TypeOperationClientInput> {
        let v: Box<dyn Any> = v;

        cast_input_to_client_case!(v, use_case_create_account_wire_client);
        cast_input_to_client_case!(v, use_case_create_account_for_branch_wire_client);
        cast_input_to_client_case!(v, use_case_create_branch_wire_client);
        cast_input_to_client_case!(v, use_case_create_company_wire_client);
        cast_input_to_client_case!(v, use_case_create_journal_wire_client);
        cast_input_to_client_case!(v, use_case_create_journal_entry_wire_client);
        cast_input_to_client_case!(v, use_case_get_all_accounts_wire_client);
        cast_input_to_client_case!(v, use_case_get_all_journals_wire_client);
        cast_input_to_client_case!(v, use_case_get_companies_and_branches_wire_client);
        cast_input_to_client_case!(v, use_case_sign_in_wire_client);
        cast_input_to_client_case!(v, use_case_sign_up_wire_client);

        bail!("downcast error in cast_input method")
    }

    fn cast_ok(v: TypeOperationDTOOk) -> Result<TypeOperationClientOk> {
        let v: Box<dyn Any> = v;

        cast_ok_to_client_case!(v, use_case_create_account_wire_client);
        cast_ok_to_client_case!(v, use_case_create_account_for_branch_wire_client);
        cast_ok_to_client_case!(v, use_case_create_branch_wire_client);
        cast_ok_to_client_case!(v, use_case_create_company_wire_client);
        cast_ok_to_client_case!(v, use_case_create_journal_wire_client);
        cast_ok_to_client_case!(v, use_case_create_journal_entry_wire_client);
        cast_ok_to_client_case!(v, use_case_get_all_accounts_wire_client);
        cast_ok_to_client_case!(v, use_case_get_all_journals_wire_client);
        cast_ok_to_client_case!(v, use_case_get_companies_and_branches_wire_client);
        cast_ok_to_client_case!(v, use_case_sign_in_wire_client);
        cast_ok_to_client_case!(v, use_case_sign_up_wire_client);

        bail!("downcast error in cast_ok method")
    }

    fn cast_error(v: TypeOperationDTOError) -> Result<TypeOperationClientError> {
        let v: Box<dyn Any> = v;

        cast_error_to_client_case!(v, use_case_create_account_wire_client);
        cast_error_to_client_case!(v, use_case_create_account_for_branch_wire_client);
        cast_error_to_client_case!(v, use_case_create_branch_wire_client);
        cast_error_to_client_case!(v, use_case_create_company_wire_client);
        cast_error_to_client_case!(v, use_case_create_journal_wire_client);
        cast_error_to_client_case!(v, use_case_create_journal_entry_wire_client);
        cast_error_to_client_case!(v, use_case_get_all_accounts_wire_client);
        cast_error_to_client_case!(v, use_case_get_all_journals_wire_client);
        cast_error_to_client_case!(v, use_case_get_companies_and_branches_wire_client);
        cast_error_to_client_case!(v, use_case_sign_in_wire_client);
        cast_error_to_client_case!(v, use_case_sign_up_wire_client);

        bail!("downcast error in cast_error method")
    }
}
