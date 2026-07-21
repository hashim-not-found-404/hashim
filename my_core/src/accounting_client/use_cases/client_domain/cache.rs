use crate::accounting_domain::{
    cases::utility::{resource_utils, types},
    request_response,
};

pub trait Cache: Sized {
    fn new() -> impl Future<Output = Self>;

    fn get_all_txn_input(
        &self,
    ) -> impl Future<
        Output = Vec<
            request_response::push_data::Txn<request_response::push_data::OperationsInput>,
        >,
    >;
    fn write_txn_input(
        &self,
        txn: &request_response::push_data::Txn<request_response::push_data::OperationsInput>,
    ) -> impl Future<Output = ()>;
    fn write_txn_result(
        &self,
        txn: &request_response::push_data::Txn<request_response::push_data::OperationsResult>,
    ) -> impl Future<Output = ()>;
    fn mark_txn_input_as_faild(&self, txn_number: &u64) -> impl Future<Output = ()>;
    fn delete_txn_input(&self, txn_number: &u64) -> impl Future<Output = ()>;

    fn write_resource(
        &self,
        resource: &Vec<resource_utils::ResourceInfo>,
    ) -> impl Future<Output = ()>;
    fn get_jwt(
        &self,
        user_uuid: &types::UuidType,
    ) -> impl Future<Output = Option<types::JsonWebTokenType>>;

    fn read_sign_in(
        &self,
        user_id: &String,
    ) -> impl Future<
        Output = Option<(
            types::UuidType, /* user uuid */
            Option<String>,  /* user name */
            bool,            /* does he have jwt */
        )>,
    >;
}

pub(crate) struct State<Ch: Cache> {
    pub(crate) state_of_pending_txn: resource_utils::StateOfPendingTxn,
    pub(crate) cache: Ch,
}
