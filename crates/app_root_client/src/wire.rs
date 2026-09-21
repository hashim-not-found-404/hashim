use patterns::make_client_wrapper_cache_check;
use patterns::make_client_wrapper_cache_write;
use patterns::make_client_wrapper_updater;
use std::any::Any;

make_client_wrapper_updater!(create_account);
make_client_wrapper_cache_check!(create_account);
make_client_wrapper_cache_write!(create_account);

make_client_wrapper_cache_check!(get_all_accounts);
make_client_wrapper_cache_write!(get_all_accounts);

use crate::model::TypeModel;
use cache::cache_adapter;
use kernel::ui_construct::CastDTOToClient;
use paste::paste;
use utility::cache::CastClientToCache;
use utility::cache::TraitOperationCacheInput;
use utility::cache::TraitOperationCacheOk;
use utility::cache::TraitOperationClientError;
use utility::cache::TraitOperationClientInput;
use utility::cache::TraitOperationClientOk;
use utility::dtos::TypeOperationDTOError;
use utility::dtos::TypeOperationDTOInput;
use utility::dtos::TypeOperationDTOOk;
use utility::ui_effect::CastMessageToUpdater;
use utility::ui_effect::MessageTrait;
use utility::ui_effect::UpdaterTrait;

// macro_rules! downcast {
//     ($v:expr, $crate_name:tt) => {
//         if let Some(v) = $v.downcast_ref::<$crate_name::>() {
//             paste! {
//                 return Box::new(crate::wire::[<updater_ $crate_name>]::Wrapper(v.clone()));
//             };
//         };
//     };
// }

pub(crate) struct MyCaster;

impl CastMessageToUpdater for MyCaster {
    type Mdl = TypeModel;

    fn cast_message_to_updater(v: Box<dyn MessageTrait>) -> Box<dyn UpdaterTrait<Mdl = Self::Mdl>> {
        let v: Box<dyn Any> = v;

        // downcast!(v, use_case_get_all_accounts);
        // downcast!(v, use_case_create_account);

        unreachable!()
    }
}

impl CastClientToCache for MyCaster {
    type Cache = cache_adapter::S;

    fn cast_input(
        v: &dyn TraitOperationClientInput,
    ) -> &dyn TraitOperationCacheInput<Cache = Self::Cache> {
        todo!()
    }

    fn cast_ok(v: &dyn TraitOperationClientOk) -> &dyn TraitOperationCacheOk<Cache = Self::Cache> {
        todo!()
    }
}

impl CastDTOToClient for MyCaster {
    fn cast_input(v: TypeOperationDTOInput) -> Box<dyn TraitOperationClientInput> {
        todo!()
    }

    fn cast_ok(v: TypeOperationDTOOk) -> Box<dyn TraitOperationClientOk> {
        todo!()
    }

    fn cast_error(v: TypeOperationDTOError) -> Box<dyn TraitOperationClientError> {
        todo!()
    }
}
