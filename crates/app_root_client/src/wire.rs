use crate::model::TypeGlobalModel;
use crate::use_case_home;
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
use patterns::make_client_wrapper_cache_check;
use patterns::make_client_wrapper_cache_write;
use patterns::make_client_wrapper_updater;
use patterns::make_client_wrapper_updater_form;
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
use utility::ui_effect::CastMessageToReducer;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UpdaterTrait;

mod updater_use_case_navigator {
    use crate::model::TypeGlobalModel;
    use crate::navigator::Change;
    use crate::navigator::Effect;
    use crate::navigator::Message;
    use crate::navigator::apply;
    use crate::navigator::effect;
    use crate::navigator::update;
    use anyhow::Result;
    use std::pin::Pin;
    use utility::cache::ProcessId;
    use utility::ui_effect::ApplierTrait;
    use utility::ui_effect::EffectorTrait;
    use utility::ui_effect::UiContext;
    use utility::ui_effect::UpdaterTrait;

    #[derive(Debug)]
    pub(crate) struct WrapperMessage(pub(crate) Message);

    impl UpdaterTrait for WrapperMessage {
        type Mdl = TypeGlobalModel;
        fn update(
            &self,
            model: &Self::Mdl,
            process_id: ProcessId,
        ) -> (
            Vec<Box<dyn ApplierTrait<Mdl = TypeGlobalModel>>>,
            Vec<Box<dyn EffectorTrait>>,
        ) {
            let (changes, effects) = update(self.0.clone(), model);

            let updaters: Vec<Box<dyn ApplierTrait<Mdl = TypeGlobalModel>>> = changes
                .into_iter()
                .map(|c| Box::new(WrapperChange(c)) as Box<dyn ApplierTrait<Mdl = TypeGlobalModel>>)
                .collect();

            let effectors: Vec<Box<dyn EffectorTrait>> = effects
                .into_iter()
                .map(|e| Box::new(WrapperEffect(e)) as Box<dyn EffectorTrait>)
                .collect();

            (updaters, effectors)
        }
    }

    #[derive(Debug)]
    pub(crate) struct WrapperChange(pub(crate) Change);

    impl ApplierTrait for WrapperChange {
        type Mdl = TypeGlobalModel;
        fn apply(&self, model: &Self::Mdl, _: ProcessId) {
            apply(self.0.clone(), model);
        }
    }

    #[derive(Debug)]
    pub(crate) struct WrapperEffect(pub(crate) Effect);

    impl EffectorTrait for WrapperEffect {
        fn effect(
            &self,
            process_id: ProcessId,
            context: UiContext,
        ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
            Box::pin(effect(self.0.clone(), context))
        }
    }
}

mod updater_use_case_home {
    use crate::model::TypeGlobalModel;
    use crate::use_case_home::client::Change;
    use crate::use_case_home::client::Effect;
    use crate::use_case_home::client::Message;
    use crate::use_case_home::client::apply;
    use crate::use_case_home::client::effect;
    use crate::use_case_home::client::update;
    use anyhow::Result;
    use std::pin::Pin;
    use utility::cache::ProcessId;
    use utility::ui_effect::ApplierTrait;
    use utility::ui_effect::EffectorTrait;
    use utility::ui_effect::UiContext;
    use utility::ui_effect::UpdaterTrait;
    #[derive(Debug)]
    pub(crate) struct WrapperMessage(pub(crate) Message);

    impl UpdaterTrait for WrapperMessage {
        type Mdl = TypeGlobalModel;
        fn update(
            &self,
            model: &Self::Mdl,
            process_id: ProcessId,
        ) -> (
            Vec<Box<dyn ApplierTrait<Mdl = TypeGlobalModel>>>,
            Vec<Box<dyn EffectorTrait>>,
        ) {
            let (changes, effects) = update(self.0.clone(), &model.page_home, model);
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
    pub(crate) struct WrapperChange(pub(crate) Change);

    impl ApplierTrait for WrapperChange {
        type Mdl = TypeGlobalModel;
        fn apply(&self, model: &Self::Mdl, process_id: ProcessId) {
            apply(self.0.clone(), &model.page_home, model);
        }
    }
    #[derive(Debug)]
    pub(crate) struct WrapperEffect(pub(crate) Effect);

    impl EffectorTrait for WrapperEffect {
        fn effect(
            &self,
            process_id: ProcessId,
            context: UiContext,
        ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
            Box::pin(effect(self.0.clone(), process_id, context))
        }
    }
}

make_client_wrapper_updater_form!(create_account);
make_client_wrapper_cache_check!(create_account);
make_client_wrapper_cache_write!(create_account);

make_client_wrapper_updater!(error_handler);

make_client_wrapper_cache_check!(get_all_accounts);
make_client_wrapper_cache_write!(get_all_accounts);

make_client_wrapper_updater!(sign_up);
make_client_wrapper_cache_check!(sign_up);
make_client_wrapper_cache_write!(sign_up);

make_client_wrapper_updater!(sign_in);
make_client_wrapper_cache_check!(sign_in);
make_client_wrapper_cache_write!(sign_in);

make_client_wrapper_updater_form!(create_company);
make_client_wrapper_cache_check!(create_company);
make_client_wrapper_cache_write!(create_company);

make_client_wrapper_updater!(select_default_company);
make_client_wrapper_cache_check!(get_companies_and_branches);
make_client_wrapper_cache_write!(get_companies_and_branches);

make_client_wrapper_updater_form!(create_branch);
make_client_wrapper_cache_check!(create_branch);
make_client_wrapper_cache_write!(create_branch);

make_client_wrapper_updater_form!(create_account_for_branch);
make_client_wrapper_cache_check!(create_account_for_branch);
make_client_wrapper_cache_write!(create_account_for_branch);

make_client_wrapper_updater_form!(create_journal_entry);
make_client_wrapper_cache_check!(create_journal_entry);
make_client_wrapper_cache_write!(create_journal_entry);

pub(crate) struct MyCaster;

impl CastMessageToReducer for MyCaster {
    type Mdl = TypeGlobalModel;

    fn cast_message_to_reducer(
        v: Box<dyn MessageTrait>,
    ) -> Result<Box<dyn UpdaterTrait<Mdl = Self::Mdl>>> {
        let v: Box<dyn Any> = v;

        if let Some(v) = v.downcast_ref::<crate::navigator::Message>() {
            return Ok(Box::new(updater_use_case_navigator::WrapperMessage(
                v.clone(),
            )));
        }

        cast_message_to_reducer_case!(home);
        cast_message_to_reducer_case!(create_account);
        cast_message_to_reducer_case!(error_handler);
        cast_message_to_reducer_case!(sign_in);
        cast_message_to_reducer_case!(sign_up);
        cast_message_to_reducer_case!(create_company);
        cast_message_to_reducer_case!(select_default_company);
        cast_message_to_reducer_case!(create_branch);
        cast_message_to_reducer_case!(create_account_for_branch);
        cast_message_to_reducer_case!(create_journal_entry);

        bail!("downcast error")
    }
}

impl CastClientToCache for MyCaster {
    type Cache = cache_adapter::S;

    fn cast_input(
        v: TypeOperationClientInput,
    ) -> Result<Box<dyn TraitOperationCacheInput<Cache = Self::Cache>>> {
        let v: Arc<dyn Any> = v;

        cast_input_to_cache_check_case!(create_account);
        cast_input_to_cache_check_case!(get_all_accounts);
        cast_input_to_cache_check_case!(sign_in);
        cast_input_to_cache_check_case!(sign_up);
        cast_input_to_cache_check_case!(create_company);
        cast_input_to_cache_check_case!(get_companies_and_branches);
        cast_input_to_cache_check_case!(create_branch);
        cast_input_to_cache_check_case!(create_account_for_branch);
        cast_input_to_cache_check_case!(create_journal_entry);

        bail!("downcast error")
    }

    fn cast_ok(
        v: TypeOperationClientOk,
    ) -> Result<Box<dyn TraitOperationCacheOk<Cache = Self::Cache>>> {
        let v: Arc<dyn Any> = v;

        cast_ok_to_cache_write_case!(create_account);
        cast_ok_to_cache_write_case!(get_all_accounts);
        cast_ok_to_cache_write_case!(sign_in);
        cast_ok_to_cache_write_case!(sign_up);
        cast_ok_to_cache_write_case!(create_company);
        cast_ok_to_cache_write_case!(get_companies_and_branches);
        cast_ok_to_cache_write_case!(create_branch);
        cast_ok_to_cache_write_case!(create_account_for_branch);
        cast_ok_to_cache_write_case!(create_journal_entry);

        bail!("downcast error")
    }
}

impl CastDTOToClient for MyCaster {
    fn cast_input(v: TypeOperationDTOInput) -> Result<TypeOperationClientInput> {
        let v: Box<dyn Any> = v;

        cast_input_to_client_case!(create_account);
        cast_input_to_client_case!(get_all_accounts);
        cast_input_to_client_case!(sign_in);
        cast_input_to_client_case!(sign_up);
        cast_input_to_client_case!(create_company);
        cast_input_to_client_case!(get_companies_and_branches);
        cast_input_to_client_case!(create_branch);
        cast_input_to_client_case!(create_account_for_branch);
        cast_input_to_client_case!(create_journal_entry);

        bail!("downcast error")
    }

    fn cast_ok(v: TypeOperationDTOOk) -> Result<TypeOperationClientOk> {
        let v: Box<dyn Any> = v;

        cast_ok_to_client_case!(create_account);
        cast_ok_to_client_case!(get_all_accounts);
        cast_ok_to_client_case!(sign_in);
        cast_ok_to_client_case!(sign_up);
        cast_ok_to_client_case!(create_company);
        cast_ok_to_client_case!(get_companies_and_branches);
        cast_ok_to_client_case!(create_branch);
        cast_ok_to_client_case!(create_account_for_branch);
        cast_ok_to_client_case!(create_journal_entry);

        bail!("downcast error")
    }

    fn cast_error(v: TypeOperationDTOError) -> Result<TypeOperationClientError> {
        let v: Box<dyn Any> = v;

        cast_error_to_client_case!(create_account);
        cast_error_to_client_case!(get_all_accounts);
        cast_error_to_client_case!(sign_in);
        cast_error_to_client_case!(sign_up);
        cast_error_to_client_case!(create_company);
        cast_error_to_client_case!(get_companies_and_branches);
        cast_error_to_client_case!(create_branch);
        cast_error_to_client_case!(create_account_for_branch);
        cast_error_to_client_case!(create_journal_entry);

        bail!("downcast error")
    }
}
