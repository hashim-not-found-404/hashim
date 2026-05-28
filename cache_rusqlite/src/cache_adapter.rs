use crate::prelude::*;
use rusqlite::{Connection, Result};

pub struct S {
    db: Connection,
}

impl CacheIO for S {
    async fn new() -> Result<Self, DynamicError> {
        let conn = Connection::open("opfs-sahpool://my_persistent_database.db").unwrap();
        Ok(Self { db: conn })
    }

    async fn get_all_write_txns(
        &self,
    ) -> Result<Vec<push_data::TxnInput<push_data::WriteOperationInput>>, DynamicError> {
        todo!()
    }

    async fn get_jwt(
        &self,
        user_uuid: &db_types::RowIdType,
    ) -> Result<Option<String>, DynamicError> {
        todo!()
    }

    async fn write_data(&self, data: &Vec<ResourceInfo>) -> Result<(), DynamicError> {
        todo!()
    }

    async fn write_txn<T>(&self, txn: &push_data::TxnInput<T>) -> Result<(), DynamicError> {
        todo!()
    }

    async fn get_txn<T>(
        &self,
        user_uuid: &db_types::RowIdType,
        txn_number: &u64,
    ) -> Result<push_data::TxnInput<T>, DynamicError> {
        todo!()
    }

    async fn delete_txn(
        &self,
        user_uuid: &db_types::RowIdType,
        txn_number: &u64,
    ) -> Result<(), DynamicError> {
        todo!()
    }
}
