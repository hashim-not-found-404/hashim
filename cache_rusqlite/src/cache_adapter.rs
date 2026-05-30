use crate::prelude::*;
use rusqlite::{Connection, Result};

pub struct S {
    db: Connection,
}

impl CacheIO for S {
    async fn new() -> Result<Self, DynamicError> {
        let conn = Connection::open("opfs-sahpool://my_persistent_database.db").unwrap();

        const SCHEMA: &str = include_str!("../schema/tables.sql");
        conn.execute_batch(SCHEMA).unwrap();

        Ok(Self { db: conn })
    }

    async fn get_all_write_txns(
        &self,
    ) -> Result<Vec<push_data::TxnInput<push_data::WriteOperationInput>>, DynamicError> {
        let mut stmt = self
            .db
            .prepare("SELECT txn_number, user_, txn FROM write_cache_write_transactions")
            .unwrap();

        let rows = stmt
            .query_map([], |row| {
                Ok(push_data::TxnInput {
                    user_uuid: row.get(1).unwrap(),
                    txn_number: row.get::<usize, i64>(0).unwrap() as u64,
                    operation: encode_decode::m::S::decode(&row.get::<usize, Vec<u8>>(2).unwrap())
                        .unwrap(),
                })
            })
            .unwrap();

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(row.unwrap());
        }
        Ok(transactions)
    }

    async fn write_auth_to_cache(
        &self,
        txn_number: &u64,
        txn: &push_data::AuthenticationMethodInput,
    ) -> Result<(), DynamicError> {
        let txn_data = encode_decode::m::S::encode(txn);
        self.db
            .execute(
                "INSERT OR REPLACE INTO write_cache_auth_transactions (txn_number, txn) VALUES (?1, ?2)",
                rusqlite::params![*txn_number as i64, txn_data],
            )
            .unwrap();

        Ok(())
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
