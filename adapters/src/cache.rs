pub mod m {
    use crate::prelude::*;
    pub struct S;

    impl CacheIO for S {
        type RowId = row_id::m::S;

        async fn new() -> Result<Self, DynamicError> {
            mbg!("here is the error was");
            Ok(S)
        }

        async fn write_data(&self, data: &Vec<ResourceInfo>) -> Result<(), DynamicError> {
            todo!()
        }

        async fn write_txn(&self, txn: &push_data::TxnInput) -> Result<(), DynamicError> {
            todo!()
        }

        async fn get_txn(
            &self,
            user_uuid: &Self::RowId,
            txn_number: &u64,
        ) -> Result<push_data::TxnInput, DynamicError> {
            todo!()
        }

        async fn delete_txn(
            &self,
            user_uuid: &Self::RowId,
            txn_number: &u64,
        ) -> Result<(), DynamicError> {
            todo!()
        }
    }
}
