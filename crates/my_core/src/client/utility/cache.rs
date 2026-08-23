use crate::domain::request_response;
use crate::domain::utility::types::JsonWebTokenType;
use crate::domain::utility::uuid::User;

pub trait Cache: Sized {
    fn new() -> impl Future<Output = Self>;

    fn get_all_txn_input(
        &self,
    ) -> impl Future<Output = Vec<request_response::Txn<request_response::OperationsInput>>>;
    fn write_txn_input(
        &self,
        txn: &request_response::Txn<request_response::OperationsInput>,
    ) -> impl Future<Output = ()>;
    fn write_txn_result(
        &self,
        txn: &request_response::Txn<request_response::OperationsResult>,
    ) -> impl Future<Output = ()>;
    fn mark_txn_input_as_faild(&self, txn_number: &u64) -> impl Future<Output = ()>;
    fn delete_txn_input(&self, txn_number: &u64) -> impl Future<Output = ()>;

    fn write_resource_from_server(
        &self,
        resource: &[resource_utils::ResourceInfo],
    ) -> impl Future<Output = ()>;
    fn write_resource_of_pending_txn(
        &self,
        resource: &[resource_utils::ResourceInfo],
    ) -> impl Future<Output = ()>;
    fn clear_pending_txn_state(&self) -> impl Future<Output = ()>;
    fn start_pending_txn_state(&self) -> impl Future<Output = ()>;

    fn get_jwt(&self, user_uuid: &User) -> impl Future<Output = Option<JsonWebTokenType>>;
}
