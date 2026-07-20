use crate::{
    accounting_client::use_cases::client_domain::{cache, cache_actor, commander, ui_model},
    accounting_domain::{
        cases::utility::{resource_utils, types},
        request_response,
    },
    utility::traits,
};
use std::sync::Arc;

pub(crate) trait ViewAndCache<Ch: cache::Cache, T> {
    type Type1;
    type Type2: Clone;
    type Type3;
    type Type4;

    fn subs() -> &'static [resource_utils::Subscribe] {
        unreachable!("we dont need it here")
    }

    fn wrap_input(data: Self::Type1) -> request_response::push_data::OperationsInput;

    fn user_uuid(data: &Self::Type2) -> Option<&types::UuidType>;

    async fn state_full_operation<Id: types::RowId>(
        data: &Self::Type2,
        state: &mut cache::State<Ch>,
    ) -> Self::Type3;

    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo>;

    fn wrap_output(data: Self::Type3) -> request_response::push_data::OperationsResult;

    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self::Type4;
}

///////////////////////////////////////////

// pub(crate) trait ViewType1 {
//     fn subs() -> &'static [resource_utils::Subscribe] {
//         unreachable!("we dont need it here")
//     }
//     fn wrap_input(self) -> request_response::push_data::OperationsInput;
// }

// pub(crate) trait CacheAndServerType1: Clone {
//     fn user_uuid(&self) -> Option<&types::UuidType>;

//     type Output: CacheAndServerType2;
//     async fn state_full_operation<Id: types::RowId, Ch: cache::Cache>(
//         &self,
//         state: &mut cache::State<Ch>,
//     ) -> Self::Output;
// }

// pub(crate) trait CacheAndServerType2 {
//     fn extract_resource(&self) -> Vec<resource_utils::ResourceInfo>;
//     fn wrap_output(self) -> request_response::push_data::OperationsResult;
// }

// pub(crate) trait ViewType2 {
//     fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self;
// }

pub(crate) type CacheActorStruct<Mpsc> = cache_actor::CacheStruct<
    Mpsc,
    resource_utils::Subscribe,
    request_response::push_data::OperationsInput,
    request_response::push_data::OperationsResult,
>;
