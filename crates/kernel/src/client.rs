// use crate::domain::request_response::OperationsInput;
// use crate::domain::request_response::OperationsResult;
// use crate::domain::request_response::Txn;
// use crate::new_types::JsonWebTokenType;
// use crate::new_types::UserUuid;

// pub trait Cache: Sized {
//     fn new() -> impl Future<Output = Self>;

//     fn get_all_txn_input(&self) -> impl Future<Output = Vec<Txn<OperationsInput>>>;
//     fn write_txn_input(&self, txn: &Txn<OperationsInput>) -> impl Future<Output = ()>;
//     fn write_txn_result(&self, txn: &Txn<OperationsResult>) -> impl Future<Output = ()>;
//     fn mark_txn_input_as_faild(&self, txn_number: &u64) -> impl Future<Output = ()>;
//     fn delete_txn_input(&self, txn_number: &u64) -> impl Future<Output = ()>;
//     fn clear_pending_txn_state(&self) -> impl Future<Output = ()>;
//     fn start_pending_txn_state(&self) -> impl Future<Output = ()>;

//     fn get_jwt(&self, user_uuid: &UserUuid) -> impl Future<Output = Option<JsonWebTokenType>>;
// }
