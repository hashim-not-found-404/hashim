use crate::utils::MyUuidConverter;
use anyhow::Result;
use infrastructure::jwt::JsonWebTokenType;
use kernel::client::Cache;
use kernel::new_types::UserUuid;
use rusqlite::Connection;
use utility::cache::MarkerCache;
use utility::dtos::Txn;
use utility::dtos::TxnNumber;

const QUERY1: &str =
    "SELECT txn_number, txn FROM write_cache_transactions_input WHERE is_faild = false";
const QUERY2: &str =
    "INSERT OR REPLACE INTO write_cache_transactions_input (txn_number, txn) VALUES (?1, ?2)";
const QUERY3: &str =
    "INSERT OR REPLACE INTO write_cache_transactions_result (txn_number, txn) VALUES (?1, ?2)";
const QUERY4: &str =
    "UPDATE write_cache_transactions_input SET is_faild = true WHERE txn_number = ?1";
const QUERY5: &str = "DELETE FROM write_cache_transactions_input WHERE txn_number = ?1";
const QUERY6: &str = "SELECT jwt FROM user WHERE rowid = ?1";
const QUERY7: &str =
    "ROLLBACK TO SAVEPOINT pending_txn_branch; RELEASE SAVEPOINT pending_txn_branch;";
const QUERY8: &str = "SAVEPOINT pending_txn_branch;";

pub struct S {
    pub tables_db: Connection,
    pub(crate) transactions_db: Connection,
}

impl MarkerCache for S {}

impl Cache for S {
    async fn new() -> Result<Self> {
        let tables_db = Connection::open("opfs-sahpool://tables.db")?;
        const TABLES_SCHEMA: &str = include_str!("../schema/tables.sql");
        tables_db.execute_batch(TABLES_SCHEMA)?;

        let transactions_db = Connection::open("opfs-sahpool://transactions.db")?;
        const TRANSACTIONS_SCHEMA: &str = include_str!("../schema/transactions.sql");
        transactions_db.execute_batch(TRANSACTIONS_SCHEMA)?;

        Ok(Self {
            tables_db,
            transactions_db,
        })
    }

    async fn get_all_pending_txn(&self) -> Result<Vec<Txn<Vec<u8>>>> {
        let mut stmt = self.transactions_db.prepare(QUERY1)?;

        let rows = stmt.query_map([], |row| {
            let txn_number = row.get::<usize, i64>(0)? as u64;
            let txn_number = TxnNumber(txn_number);
            let items = row.get::<usize, Vec<u8>>(1)?;
            Ok(Txn {
                txn_number,
                operation: items,
            })
        })?;

        let mut transactions: Vec<Txn<Vec<u8>>> = Vec::new();
        for row in rows {
            transactions.push(row?);
        }
        Ok(transactions)
    }

    async fn write_txn_input(&self, txn: &Txn<Vec<u8>>) -> Result<()> {
        let txn_data = txn.operation.clone();
        self.transactions_db
            .execute(QUERY2, rusqlite::params![txn.txn_number.0 as i64, txn_data])?;

        Ok(())
    }

    async fn write_txn_result(&self, txn: &Txn<Vec<u8>>) -> Result<()> {
        let txn_data = txn.operation.clone();
        self.transactions_db
            .execute(QUERY3, rusqlite::params![txn.txn_number.0 as i64, txn_data])?;

        Ok(())
    }

    async fn mark_input_txn_as_faild(&self, txn_number: TxnNumber) -> Result<()> {
        self.transactions_db
            .execute(QUERY4, rusqlite::params![txn_number.0 as i64])?;

        Ok(())
    }

    async fn delete_input_txn(&self, txn_number: TxnNumber) -> Result<()> {
        self.transactions_db
            .execute(QUERY5, rusqlite::params![txn_number.0 as i64])?;

        Ok(())
    }

    async fn clear_pending_txn_state(&self) -> Result<()> {
        self.tables_db.execute_batch(QUERY7)?;

        Ok(())
    }

    async fn start_pending_txn_state(&self) -> Result<()> {
        self.tables_db.execute_batch(QUERY8)?;

        Ok(())
    }

    async fn get_jwt(&self, user_uuid: &UserUuid) -> Result<Option<JsonWebTokenType>> {
        let mut stmt = self.tables_db.prepare(QUERY6)?;

        let json_web_token_type =
            stmt.query_one([&user_uuid.to_string()], |row| row.get::<_, String>(0));

        let json_web_token = match json_web_token_type {
            Ok(a) => Some(JsonWebTokenType::from(a)),
            Err(_) => None,
        };

        Ok(json_web_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::test_query_helper_for_tables_schema;
    use crate::test_helper::test_query_helper_for_transactions_schema;

    #[test]
    fn test_query_string_directly() {
        test_query_helper_for_transactions_schema(QUERY1).unwrap();
        test_query_helper_for_transactions_schema(QUERY2).unwrap();
        test_query_helper_for_transactions_schema(QUERY3).unwrap();
        test_query_helper_for_transactions_schema(QUERY4).unwrap();
        test_query_helper_for_transactions_schema(QUERY5).unwrap();
        test_query_helper_for_tables_schema(QUERY6).unwrap();
        test_query_helper_for_transactions_schema(format!("{}{}", QUERY8, QUERY7).as_str())
            .unwrap();
        test_query_helper_for_transactions_schema(QUERY8).unwrap();
    }
}
