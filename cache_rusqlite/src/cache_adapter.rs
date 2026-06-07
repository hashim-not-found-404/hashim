use crate::prelude::*;
use rusqlite::{Connection, OptionalExtension, params};

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

    async fn write_resource(&self, resource: &Vec<ResourceInfo>) {
        let mut stmts = Vec::with_capacity(resource.len());

        for reso in resource {
            let uuid = &reso.uuid.0;

            let stmt = match &reso.resource {
                server_methods::Resource::Jwt(value) => {
                    make_sql_statment("user", "jwt", uuid, value)
                }
                server_methods::Resource::UserName(value) => {
                    make_sql_statment("user", "name", uuid, value)
                }
                server_methods::Resource::UserId(value) => {
                    make_sql_statment("user", "id", uuid, value)
                }
                server_methods::Resource::CompanyName(value) => {
                    make_sql_statment("company", "name", uuid, value)
                }
                server_methods::Resource::CompanyCurrency(value) => {
                    make_sql_statment("company", "currency", uuid, &value.as_str().to_string())
                }
                server_methods::Resource::RoleAtCompany(value) => {
                    todo!();
                    make_sql_statment("", "", uuid, &value.as_str().to_string())
                }
                server_methods::Resource::UserThatHaveRole(value) => {
                    todo!();
                    make_sql_statment("", "", uuid, &value.0)
                }
            };

            stmts.push(stmt);
        }

        let stmts = stmts.concat();
        self.db.execute_batch(stmts.as_str()).unwrap();
    }

    async fn get_jwt(&self, user_uuid: &db_types::UuidType) -> Option<String> {
        let mut stmt = self
            .db
            .prepare("SELECT jwt FROM user WHERE row_id = ?1")
            .unwrap();

        stmt.query_one([&user_uuid.0], |row| row.get(0).optional())
            .unwrap()
    }

    async fn read_sign_up(
        &self,
        new_uuid: &db_types::UuidType,
        user_id: &String,
    ) -> (
        bool, /* is new_uuid exist */
        bool, /* is user_id exist */
    ) {
        let query = "
            SELECT
                EXISTS(SELECT 1 FROM user WHERE rowid = ?1),
                EXISTS(SELECT 1 FROM user WHERE id = ?2)
        ";

        self.db
            .query_one(query, params![new_uuid.0, user_id], |row| {
                Ok((row.get(0).unwrap(), row.get(1).unwrap()))
            })
            .unwrap()
    }

    async fn read_get_user_uuid(&self, user_id: &String) -> Option<db_types::UuidType> {
        let query = "SELECT 1 FROM user WHERE id = ?1";

        let user_uuid: Option<String> = self
            .db
            .query_one(query, params![user_id], |row| Ok(row.get(0).unwrap()))
            .optional()
            .unwrap();

        match user_uuid {
            Some(user_uuid) => Some(db_types::UuidType(user_uuid)),
            None => None,
        }
    }
}

fn make_sql_statment(table_name: &str, field_name: &str, uuid: &String, value: &String) -> String {
    format!(
        "INSERT OR REPLACE INTO {table_name} (rowid, {field_name}) VALUES ('{uuid}', '{value}');"
    )
}
