use crate::mcp_home_forms;
use crate::model::TypeGlobalModel;
use crate::use_case_navigator;
use anyhow::Result;
use anyhow::bail;
use cache::cache_adapter;
use kernel::ui_construct::CastDTOToClient;
use patterns::cast_error_to_client_case;
use patterns::cast_input_to_cache_check_case;
use patterns::cast_input_to_client_case;
use patterns::cast_message_to_reducer_case;
use patterns::cast_ok_to_cache_write_case;
use patterns::cast_ok_to_client_case;
use patterns::make_client_wrapper_cache;
use patterns::make_client_wrapper_form;
use patterns::make_client_wrapper_updater;
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

    impl UpdaterTrait for WrapperMessage {
        type Mdl = TypeGlobalModel;
        fn update(
            &self,
            model: &Self::Mdl,
        ) -> (
            Vec<Box<dyn ApplierTrait<Mdl = TypeGlobalModel>>>,
            Vec<Box<dyn EffectorTrait>>,
        ) {
            let (changes, effects) = update(self.0.clone(), model);
            let appliers: Vec<Box<dyn ApplierTrait<Mdl = TypeGlobalModel>>> = changes
                .into_iter()
                .map(|c| Box::new(WrapperChange(c)) as Box<dyn ApplierTrait<Mdl = TypeGlobalModel>>)
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

    impl ApplierTrait for WrapperChange {
        type Mdl = TypeGlobalModel;
        fn apply(&self, model: &mut Self::Mdl) {
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

make_client_wrapper_cache!(create_account);
make_client_wrapper_cache!(create_account_for_branch);
make_client_wrapper_cache!(create_branch);
make_client_wrapper_cache!(create_company);
make_client_wrapper_cache!(create_journal);
make_client_wrapper_cache!(create_journal_entry);
make_client_wrapper_cache!(get_all_accounts);
make_client_wrapper_cache!(get_companies_and_branches);
make_client_wrapper_cache!(sign_in);
make_client_wrapper_cache!(sign_up);
make_client_wrapper_form!(create_account);
make_client_wrapper_form!(create_account_for_branch);
make_client_wrapper_form!(create_branch);
make_client_wrapper_form!(create_company);
make_client_wrapper_form!(create_journal);
make_client_wrapper_updater!(error_handler);
make_client_wrapper_updater!(home);
make_client_wrapper_updater!(select_default_company);
make_client_wrapper_updater!(sign_in);
make_client_wrapper_updater!(sign_up);
make_mcp_provider!(home);
make_mcp_provider_form!(create_branch);
make_mcp_provider_form!(create_company);
make_mcp_provider_form!(create_journal);

pub(crate) fn get_all_providers() -> Vec<Box<dyn TraitMcpProvider>> {
    vec![
        Box::new(mcp_provider_home::TypeMcpProvider),
        Box::new(mcp_provider_create_company::TypeMcpProvider),
        Box::new(mcp_provider_create_branch::TypeMcpProvider),
        Box::new(mcp_provider_create_journal::TypeMcpProvider),
        Box::new(mcp_home_forms::TypeMcpProvider),
    ]
}

pub(crate) struct MyCaster;

impl CastMessageToReducer for MyCaster {
    type Mdl = TypeGlobalModel;

    fn cast_message_to_reducer(
        v: Box<dyn MessageTrait>,
    ) -> Result<Box<dyn UpdaterTrait<Mdl = Self::Mdl>>> {
        let v: Box<dyn Any> = v;

        cast_message_to_reducer_case!(error_handler);
        cast_message_to_reducer_case!(home);
        cast_message_to_reducer_case!(navigator);
        cast_message_to_reducer_case!(select_default_company);
        cast_message_to_reducer_case!(sign_in);
        cast_message_to_reducer_case!(sign_up);

        bail!("downcast error in cast_message_to_reducer method")
    }
}

impl CastClientToCache for MyCaster {
    type Cache = cache_adapter::S;

    fn cast_input(
        v: TypeOperationClientInput,
    ) -> Result<Box<dyn TraitOperationCacheInput<Cache = Self::Cache>>> {
        let v: Arc<dyn Any> = v;

        cast_input_to_cache_check_case!(create_account);
        cast_input_to_cache_check_case!(create_account_for_branch);
        cast_input_to_cache_check_case!(create_branch);
        cast_input_to_cache_check_case!(create_company);
        cast_input_to_cache_check_case!(create_journal);
        cast_input_to_cache_check_case!(create_journal_entry);
        cast_input_to_cache_check_case!(get_all_accounts);
        cast_input_to_cache_check_case!(get_companies_and_branches);
        cast_input_to_cache_check_case!(sign_in);
        cast_input_to_cache_check_case!(sign_up);

        bail!("downcast error in cast_input method")
    }

    fn cast_ok(
        v: TypeOperationClientOk,
    ) -> Result<Box<dyn TraitOperationCacheOk<Cache = Self::Cache>>> {
        let v: Arc<dyn Any> = v;

        cast_ok_to_cache_write_case!(create_account);
        cast_ok_to_cache_write_case!(create_account_for_branch);
        cast_ok_to_cache_write_case!(create_branch);
        cast_ok_to_cache_write_case!(create_company);
        cast_ok_to_cache_write_case!(create_journal);
        cast_ok_to_cache_write_case!(create_journal_entry);
        cast_ok_to_cache_write_case!(get_all_accounts);
        cast_ok_to_cache_write_case!(get_companies_and_branches);
        cast_ok_to_cache_write_case!(sign_in);
        cast_ok_to_cache_write_case!(sign_up);

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
        cast_error_to_client_case!(get_companies_and_branches);
        cast_error_to_client_case!(sign_in);
        cast_error_to_client_case!(sign_up);

        bail!("downcast error in cast_error method")
    }
}
