use crate::accounting_client::client_domain::cache;
use crate::accounting_client::client_domain::cache_actor;
use crate::accounting_client::client_domain::ui_model;
use crate::accounting_domain::request_response;
use crate::accounting_domain::utility::resource_utils;
use crate::accounting_domain::utility::types;

pub(crate) trait ViewAndCache<Ch: cache::Cache, T> {
    type Type1;
    type Type2;
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

    fn unwrap_output(output: request_response::push_data::OperationsResult) -> Self::Type4;

    fn apply_on_the_model<As: ui_model::AllSignalTypes>(
        output: &Self::Type4,
        model: &ui_model::Model<As>,
    );
}

pub(crate) trait ReadServerOnly {
    type Type1;
    type Type2;
    type Type3;

    fn wrap_input(data: Self::Type1) -> request_response::push_data::OperationsInput;
    fn user_uuid(data: &Self::Type2) -> Option<&types::UuidType>;
    fn extract_resource(data: &Self::Type3) -> Vec<resource_utils::ResourceInfo>;
}

pub(crate) type CacheActorStruct<Mpsc> = cache_actor::CacheStruct<
    Mpsc,
    resource_utils::Subscribe,
    request_response::push_data::OperationsInput,
    request_response::push_data::OperationsResult,
>;
