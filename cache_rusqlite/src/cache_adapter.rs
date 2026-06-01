use crate::prelude::*;
use rusqlite::Connection;

pub struct S {
    db: Connection,
}

impl CacheIO for S {
    async fn new() -> Self {
        let conn = Connection::open("opfs-sahpool://my_persistent_database.db").unwrap();

        const SCHEMA: &str = include_str!("../schema/tables.sql");
        conn.execute_batch(SCHEMA).unwrap();

        Self { db: conn }
    }

    async fn get_all_txn_input(&self) -> Vec<push_data::Txn<push_data::OperationsInput>> {
        let mut stmt = self
            .db
            .prepare("SELECT txn_number, txn FROM write_cache_transactions_input")
            .unwrap();

        let rows = stmt
            .query_map([], |row| {
                Ok(push_data::Txn {
                    txn_number: row.get::<usize, i64>(0).unwrap() as u64,
                    operation: encode_decode::m::S::decode(&row.get::<usize, Vec<u8>>(1).unwrap())
                        .unwrap(),
                })
            })
            .unwrap();

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(row.unwrap());
        }
        transactions
    }

    async fn write_txn_input(&self, txn: &push_data::Txn<push_data::OperationsInput>) -> () {
        let txn_data = encode_decode::m::S::encode(&txn.operation);
        self.db
            .execute(
                "INSERT OR REPLACE INTO write_cache_transactions_input (txn_number, txn) VALUES (?1, ?2)",
                rusqlite::params![txn.txn_number as i64, txn_data],
            )
            .unwrap();
    }

    async fn write_txn_result(&self, txn: &push_data::Txn<push_data::OperationsResult>) {
        let txn_data = encode_decode::m::S::encode(&txn.operation);
        self.db
            .execute(
                "INSERT OR REPLACE INTO write_cache_transactions_result (txn_number, txn) VALUES (?1, ?2)",
                rusqlite::params![txn.txn_number as i64, txn_data],
            )
            .unwrap();
    }

    async fn delete_txn_input(&self, txn_number: &u64) {
        self.db
            .execute(
                "DELETE FROM write_cache_transactions_input WHERE txn_number = ?1",
                rusqlite::params![*txn_number as i64],
            )
            .unwrap();
    }

    async fn get_jwt(&self, user_uuid: &db_types::RowIdType) -> Option<String> {
        todo!()
    }
}
