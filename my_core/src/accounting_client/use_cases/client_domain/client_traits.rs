use crate::{
    accounting_client::use_cases::client_domain::{cache, cache_actor, commander, ui_model},
    accounting_domain::{
        cases::utility::{resource_utils, types},
        request_response,
    },
    utility::traits,
};
use std::sync::Arc;

pub(crate) trait ViewType1 {
    fn subs() -> &'static [resource_utils::Subscribe] {
        unreachable!("we dont need it here")
    }
    fn wrap_input(self) -> request_response::push_data::OperationsInput;
}

pub(crate) trait CacheAndServerType1: Clone {
    fn user_uuid(&self) -> Option<&types::UuidType>;

    type Output: CacheAndServerType2;
    async fn state_full_operation<Id: types::RowId, Ch: cache::Cache>(
        &self,
        state: &mut cache::State<Ch>,
    ) -> Self::Output;
}

pub(crate) trait CacheAndServerType2 {
    fn extract_resource(&self) -> Vec<resource_utils::ResourceInfo>;
    fn wrap_output(self) -> request_response::push_data::OperationsResult;
}

pub(crate) trait ViewType2 {
    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self;
}

pub(crate) type CacheActorStruct<Mpsc> = cache_actor::CacheStruct<
    Mpsc,
    resource_utils::Subscribe,
    request_response::push_data::OperationsInput,
    request_response::push_data::OperationsResult,
>;

pub(crate) trait Mvu {
    async fn update<
        Rn: traits::RandomNumber,
        Rt: traits::Runtime,
        Id: types::RowId,
        Mpsc: traits::MultiProducerSingleConsumer,
        Rg: traits::Regex,
        As: ui_model::AllSignalTypes,
    >(
        self,
        model: &'static ui_model::Model<As>,
        cache: CacheActorStruct<Mpsc>,
        commander_local_state: Arc<commander::CommanderLocalState<Mpsc, As>>,
    );
}
