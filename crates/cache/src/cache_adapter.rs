use crate::utils::MyUuidConverter;
use infrastructure::jwt::JsonWebTokenType;
use kernel::client::Cache;
use kernel::new_types::UserUuid;
use rusqlite::Connection;
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
    pub(crate) tables_db:       Connection,
    pub(crate) transactions_db: Connection,
}

impl Cache for S {
    async fn new() -> Self {
        let tables_db = Connection::open("opfs-sahpool://tables.db").unwrap();
        const TABLES_SCHEMA: &str = include_str!("../schema/tables.sql");
        tables_db.execute_batch(TABLES_SCHEMA).unwrap();

        let transactions_db = Connection::open("opfs-sahpool://transactions.db").unwrap();
        const TRANSACTIONS_SCHEMA: &str = include_str!("../schema/transactions.sql");
        transactions_db.execute_batch(TRANSACTIONS_SCHEMA).unwrap();

        Self {
            tables_db,
            transactions_db,
        }
    }

    async fn get_all_pending_txn(&self) -> Vec<Txn<Vec<u8>>> {
        let mut stmt = self.transactions_db.prepare(QUERY1).unwrap();

        let rows = stmt
            .query_map([], |row| {
                let txn_number = row.get::<usize, i64>(0).unwrap() as u64;
                let txn_number = TxnNumber(txn_number);
                let items = row.get::<usize, Vec<u8>>(1).unwrap();
                Ok(Txn {
                    txn_number,
                    operation: items,
                })
            })
            .unwrap();

        let mut transactions: Vec<Txn<Vec<u8>>> = Vec::new();
        for row in rows {
            transactions.push(row.unwrap());
        }
        transactions
    }

    async fn write_txn_input(&self, txn: &Txn<Vec<u8>>) -> () {
        let txn_data = txn.operation.clone();
        self.transactions_db
            .execute(QUERY2, rusqlite::params![txn.txn_number.0 as i64, txn_data])
            .unwrap();
    }

    async fn write_txn_result(&self, txn: &Txn<Vec<u8>>) {
        let txn_data = txn.operation.clone();
        self.transactions_db
            .execute(QUERY3, rusqlite::params![txn.txn_number.0 as i64, txn_data])
            .unwrap();
    }

    async fn mark_input_txn_as_faild(&self, txn_number: TxnNumber) {
        self.transactions_db.execute(QUERY4, rusqlite::params![txn_number.0 as i64]).unwrap();
    }

    async fn delete_input_txn(&self, txn_number: TxnNumber) {
        self.transactions_db.execute(QUERY5, rusqlite::params![txn_number.0 as i64]).unwrap();
    }

    async fn clear_pending_txn_state(&self) {
        self.tables_db.execute_batch(QUERY7).unwrap();
    }

    async fn start_pending_txn_state(&self) {
        self.tables_db.execute_batch(QUERY8).unwrap();
    }

    async fn get_jwt(&self, user_uuid: &UserUuid) -> Option<JsonWebTokenType> {
        let mut stmt = self.tables_db.prepare(QUERY6).unwrap();

        let json_web_token_type =
            stmt.query_one([&user_uuid.to_string()], |row| row.get::<_, String>(0));

        match json_web_token_type {
            Ok(a) => Some(JsonWebTokenType::from(a)),
            Err(_) => None,
        }
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
