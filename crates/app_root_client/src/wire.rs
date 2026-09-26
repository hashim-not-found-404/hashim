use anyhow::Result;
use anyhow::bail;
use patterns::make_client_wrapper_cache_check;
use patterns::make_client_wrapper_cache_write;
use patterns::make_client_wrapper_updater;
use std::any::Any;
use std::sync::Arc;
use utility::cache::TypeOperationClientError;
use utility::cache::TypeOperationClientInput;
use utility::cache::TypeOperationClientOk;

make_client_wrapper_updater!(create_account);
make_client_wrapper_cache_check!(create_account);
make_client_wrapper_cache_write!(create_account);

make_client_wrapper_updater!(error_handler);

make_client_wrapper_cache_check!(get_all_accounts);
make_client_wrapper_cache_write!(get_all_accounts);

make_client_wrapper_updater!(sign_up);
make_client_wrapper_cache_check!(sign_up);
make_client_wrapper_cache_write!(sign_up);

use crate::model::TypeModel;
use cache::cache_adapter;
use kernel::ui_construct::CastDTOToClient;
use utility::cache::CastClientToCache;
use utility::cache::TraitOperationCacheInput;
use utility::cache::TraitOperationCacheOk;
use utility::dtos::TypeOperationDTOError;
use utility::dtos::TypeOperationDTOInput;
use utility::dtos::TypeOperationDTOOk;
use utility::ui_effect::CastMessageToUpdater;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UpdaterTrait;

pub(crate) struct MyCaster;

impl CastMessageToUpdater for MyCaster {
    type Mdl = TypeModel;

    fn cast_message_to_updater(
        v: Box<dyn MessageTrait>,
    ) -> Result<Box<dyn UpdaterTrait<Mdl = Self::Mdl>>> {
        let v: Box<dyn Any> = v;

        if let Some(v) = v.downcast_ref::<use_case_create_account::client::Message>() {
            return Ok(Box::new(updater_use_case_create_account::Wrapper(
                v.clone(),
            )));
        };
        if let Some(v) = v.downcast_ref::<use_case_error_handler::client::Message>() {
            return Ok(Box::new(updater_use_case_error_handler::Wrapper(v.clone())));
        };
        if let Some(v) = v.downcast_ref::<use_case_sign_up::client::Message>() {
            return Ok(Box::new(updater_use_case_sign_up::Wrapper(v.clone())));
        };

        bail!("downcast error")
    }
}

impl CastClientToCache for MyCaster {
    type Cache = cache_adapter::S;

    fn cast_input(
        v: TypeOperationClientInput,
    ) -> Result<Box<dyn TraitOperationCacheInput<Cache = Self::Cache>>> {
        let v: Arc<dyn Any> = v;

        if let Some(v) = v.downcast_ref::<use_case_create_account::domain::Input>() {
            return Ok(Box::new(cache_check_use_case_create_account::Wrapper(
                v.clone(),
            )));
        };
        if let Some(v) = v.downcast_ref::<use_case_get_all_accounts::domain::Input>() {
            return Ok(Box::new(cache_check_use_case_get_all_accounts::Wrapper(
                v.clone(),
            )));
        };
        if let Some(v) = v.downcast_ref::<use_case_sign_up::domain::Input>() {
            return Ok(Box::new(cache_check_use_case_sign_up::Wrapper(v.clone())));
        };

        bail!("downcast error")
    }

    fn cast_ok(
        v: TypeOperationClientOk,
    ) -> Result<Box<dyn TraitOperationCacheOk<Cache = Self::Cache>>> {
        let v: Arc<dyn Any> = v;

        if let Some(v) = v.downcast_ref::<use_case_create_account::domain::Ok>() {
            return Ok(Box::new(cache_write_use_case_create_account::Wrapper(
                v.clone(),
            )));
        };
        if let Some(v) = v.downcast_ref::<use_case_get_all_accounts::domain::Ok>() {
            return Ok(Box::new(cache_write_use_case_get_all_accounts::Wrapper(
                v.clone(),
            )));
        };
        if let Some(v) = v.downcast_ref::<use_case_sign_up::domain::Ok>() {
            return Ok(Box::new(cache_write_use_case_sign_up::Wrapper(v.clone())));
        };

        bail!("downcast error")
    }
}

impl CastDTOToClient for MyCaster {
    fn cast_input(v: TypeOperationDTOInput) -> Result<TypeOperationClientInput> {
        let v: Box<dyn Any> = v;

        if let Some(v) = v.downcast_ref::<use_case_create_account::domain::Input>() {
            return Ok(Arc::new(v.clone()));
        };
        if let Some(v) = v.downcast_ref::<use_case_get_all_accounts::domain::Input>() {
            return Ok(Arc::new(v.clone()));
        };
        if let Some(v) = v.downcast_ref::<use_case_sign_up::domain::Input>() {
            return Ok(Arc::new(v.clone()));
        };

        bail!("downcast error")
    }

    fn cast_ok(v: TypeOperationDTOOk) -> Result<TypeOperationClientOk> {
        let v: Box<dyn Any> = v;

        if let Some(v) = v.downcast_ref::<use_case_create_account::domain::Ok>() {
            return Ok(Arc::new(v.clone()));
        };
        if let Some(v) = v.downcast_ref::<use_case_get_all_accounts::domain::Ok>() {
            return Ok(Arc::new(v.clone()));
        };
        if let Some(v) = v.downcast_ref::<use_case_sign_up::domain::Ok>() {
            return Ok(Arc::new(v.clone()));
        };

        bail!("downcast error")
    }

    fn cast_error(v: TypeOperationDTOError) -> Result<TypeOperationClientError> {
        let v: Box<dyn Any> = v;

        if let Some(v) = v.downcast_ref::<use_case_create_account::domain::Error>() {
            return Ok(Box::new(v.clone()));
        };
        if let Some(v) = v.downcast_ref::<use_case_get_all_accounts::domain::Error>() {
            return Ok(Box::new(v.clone()));
        };
        if let Some(v) = v.downcast_ref::<use_case_sign_up::domain::Error>() {
            return Ok(Box::new(v.clone()));
        };

        bail!("downcast error")
    }
}
