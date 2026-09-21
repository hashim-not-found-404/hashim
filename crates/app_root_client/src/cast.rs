use crate::model::TypeModel;
use cache::cache_adapter;
use kernel::ui_construct::CastDTOToClient;
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

pub(crate) struct MyCaster;

impl CastMessageToUpdater for MyCaster {
    type Mdl = TypeModel;

    fn cast_message_to_updater(v: Box<dyn MessageTrait>) -> Box<dyn UpdaterTrait<Mdl = Self::Mdl>> {
        todo!()
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
