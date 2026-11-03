use crate::mcp_home_forms;
use crate::model::TypeGlobalModel;
use crate::use_case_navigator;
use anyhow::Result;
use anyhow::bail;
use cache::cache_adapter::CacheAdapter;
use kernel::ui_construct::CastDTOToClient;
use patterns::cast_error_to_client_case;
use patterns::cast_input_to_client_case;
use patterns::cast_message_to_reducer_case;
use patterns::cast_ok_to_client_case;
use patterns::make_client_wrapper_form;
use patterns::make_mcp_provider;
use patterns::make_mcp_provider_form;
use std::any::Any;
use std::sync::Arc;
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

make_client_wrapper_form!(create_account);
make_client_wrapper_form!(create_account_for_branch);
make_client_wrapper_form!(create_branch);
make_client_wrapper_form!(create_company);
make_client_wrapper_form!(create_journal);
make_mcp_provider!(home);
make_mcp_provider_form!(create_branch);
make_mcp_provider_form!(create_company);
make_mcp_provider_form!(create_journal);

pub(crate) fn get_all_providers() -> Vec<Box<dyn TraitMcpProvider>> {
    vec![
        Box::new(mcp_home_forms::TypeMcpProvider),
        Box::new(mcp_provider_create_branch::TypeMcpProvider),
        Box::new(mcp_provider_create_company::TypeMcpProvider),
        Box::new(mcp_provider_create_journal::TypeMcpProvider),
        Box::new(mcp_provider_home::TypeMcpProvider),
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

        cast_message_to_reducer_case!(navigator);

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

        cast_input_to_client_case!(create_account);
        cast_input_to_client_case!(create_account_for_branch);
        cast_input_to_client_case!(create_branch);
        cast_input_to_client_case!(create_company);
        cast_input_to_client_case!(create_journal);
        cast_input_to_client_case!(create_journal_entry);
        cast_input_to_client_case!(get_all_accounts);
        cast_input_to_client_case!(get_all_journals);
        cast_input_to_client_case!(get_companies_and_branches);
        cast_input_to_client_case!(sign_in);
        cast_input_to_client_case!(sign_up);

        bail!("downcast error in cast_input method")
    }

    fn cast_ok(v: TypeOperationDTOOk) -> Result<TypeOperationClientOk> {
        let v: Box<dyn Any> = v;

        cast_ok_to_client_case!(create_account);
        cast_ok_to_client_case!(create_account_for_branch);
        cast_ok_to_client_case!(create_branch);
        cast_ok_to_client_case!(create_company);
        cast_ok_to_client_case!(create_journal);
        cast_ok_to_client_case!(create_journal_entry);
        cast_ok_to_client_case!(get_all_accounts);
        cast_ok_to_client_case!(get_all_journals);
        cast_ok_to_client_case!(get_companies_and_branches);
        cast_ok_to_client_case!(sign_in);
        cast_ok_to_client_case!(sign_up);

        bail!("downcast error in cast_ok method")
    }

    fn cast_error(v: TypeOperationDTOError) -> Result<TypeOperationClientError> {
        let v: Box<dyn Any> = v;

        cast_error_to_client_case!(create_account);
        cast_error_to_client_case!(create_account_for_branch);
        cast_error_to_client_case!(create_branch);
        cast_error_to_client_case!(create_company);
        cast_error_to_client_case!(create_journal);
        cast_error_to_client_case!(create_journal_entry);
        cast_error_to_client_case!(get_all_accounts);
        cast_error_to_client_case!(get_all_journals);
        cast_error_to_client_case!(get_companies_and_branches);
        cast_error_to_client_case!(sign_in);
        cast_error_to_client_case!(sign_up);

        bail!("downcast error in cast_error method")
    }
}
