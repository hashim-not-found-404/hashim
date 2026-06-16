use std::str::FromStr;

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

    async fn get_all_txn_input(&self) -> Vec<push_data::Txn<operations::Input>> {
        let mut stmt = self
            .db
            .prepare(
                "SELECT txn_number, txn FROM write_cache_transactions_input WHERE is_faild = false",
            )
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

    async fn write_txn_input(&self, txn: &push_data::Txn<operations::Input>) -> () {
        let txn_data = encode_decode::m::S::encode(&txn.operation);
        self.db
            .execute(
                "INSERT OR REPLACE INTO write_cache_transactions_input (txn_number, txn) VALUES (?1, ?2)",
                rusqlite::params![txn.txn_number as i64, txn_data],
            )
            .unwrap();
    }

    async fn write_txn_result(&self, txn: &push_data::Txn<operations::Output>) {
        let txn_data = encode_decode::m::S::encode(&txn.operation);
        self.db
            .execute(
                "INSERT OR REPLACE INTO write_cache_transactions_result (txn_number, txn) VALUES (?1, ?2)",
                rusqlite::params![txn.txn_number as i64, txn_data],
            )
            .unwrap();
    }

    async fn mark_txn_input_as_faild(&self, txn_number: &u64) {
        self.db
            .execute(
                "UPDATE write_cache_transactions_input SET is_faild = true WHERE txn_number = ?1",
                rusqlite::params![*txn_number as i64],
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
            let uuid = &reso.row_uuid.0;

            let stmt = match &reso.resource {
                server_methods::Resource::Jwt(value) => {
                    make_sql_statment("user", "jwt", uuid, value)
                }
                server_methods::Resource::TableUserFieldName(value) => {
                    make_sql_statment("user", "name", uuid, value)
                }
                server_methods::Resource::TableUserFieldId(value) => {
                    make_sql_statment("user", "id", uuid, value)
                }
                server_methods::Resource::TableCompanyFieldName(value) => {
                    make_sql_statment("company", "name", uuid, value)
                }
                server_methods::Resource::TableCompanyBranchFieldName(value) => {
                    make_sql_statment("company_branch", "name", uuid, value)
                }
                server_methods::Resource::TableCompanyBranchFieldCompanyBelong(value) => {
                    make_sql_statment("company_branch", "company_belong", uuid, &value.0)
                }
                server_methods::Resource::TableCompanyFieldCurrency(value) => {
                    make_sql_statment("company", "currency", uuid, &value.as_str().to_string())
                }
                server_methods::Resource::TableAccessControlForCompanyFieldRole(value) => {
                    make_sql_statment(
                        "access_control_for_company",
                        "role",
                        uuid,
                        &value.as_str().to_string(),
                    )
                }
                server_methods::Resource::TableAccessControlForCompanyFieldUser(value) => {
                    make_sql_statment("access_control_for_company", "user_", uuid, &value.0)
                }
                server_methods::Resource::TableAccessControlForCompanyFieldDataGroup(value) => {
                    make_sql_statment("access_control_for_company", "data_group", uuid, &value.0)
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
            .prepare("SELECT jwt FROM user WHERE rowid = ?1")
            .unwrap();

        stmt.query_one([&user_uuid.0], |row| row.get(0)).ok()
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

    async fn read_sign_in(
        &self,
        user_id: &String,
    ) -> Option<(
        db_types::UuidType, /* user uuid */
        Option<String>,     /* user name */
        bool,               /* does he have jwt */
    )> {
        let query = "SELECT rowid, name, jwt FROM user WHERE id = ?1;";

        self.db
            .query_row(query, params![user_id], |row| {
                let user_uuid_str: String = row.get(0).unwrap();
                let user_name: Option<String> = row.get(1).unwrap();
                let jwt: Option<String> = row.get(2).unwrap();

                Ok((
                    db_types::UuidType(user_uuid_str),
                    user_name,
                    jwt.is_some(), // true if JWT exists
                ))
            })
            .optional()
            .unwrap()
    }

    async fn read_list_company_and_branch(
        &self,
        user_uuid: &db_types::UuidType,
    ) -> Vec<db_types::Company> {
        let query = "
            SELECT
                c.rowid as company_uuid,
                c.name as company_name,
                acf.role as user_role,
                cb.rowid as branch_uuid,
                cb.name as branch_name
            FROM access_control_for_company acf
            JOIN company c ON acf.data_group = c.rowid
            LEFT JOIN company_branch cb ON c.rowid = cb.company_belong
            WHERE acf.user_ = ?1
            ORDER BY c.rowid, cb.rowid
        ";

        let mut stmt = self.db.prepare(query).unwrap();
        let rows = stmt
            .query_map(params![user_uuid.0], |row| {
                let company_uuid_str: String = row.get(0)?;
                let company_name: String = row.get(1)?;
                let user_role_str: String = row.get(2)?;
                let branch_uuid_opt: Option<String> = row.get(3)?;
                let branch_name_opt: Option<String> = row.get(4)?;

                Ok((
                    company_uuid_str,
                    company_name,
                    user_role_str,
                    branch_uuid_opt,
                    branch_name_opt,
                ))
            })
            .unwrap();

        let mut companies: Vec<db_types::Company> = Vec::new();
        let mut current_company_uuid: Option<String> = None;

        for row in rows {
            let (company_uuid_str, company_name, user_role_str, branch_uuid_opt, branch_name_opt) =
                row.unwrap();

            if current_company_uuid.as_ref() != Some(&company_uuid_str) {
                current_company_uuid = Some(company_uuid_str.clone());

                let role = db_types::Role::from_str(user_role_str.as_str()).unwrap();

                let new_company = db_types::Company {
                    uuid: db_types::UuidType(company_uuid_str),
                    name: company_name,
                    role,
                    branches: Vec::new(),
                };
                companies.push(new_company);
            }

            if let (Some(branch_uuid_str), Some(branch_name)) = (branch_uuid_opt, branch_name_opt) {
                let branch = db_types::Branch {
                    uuid: db_types::UuidType(branch_uuid_str),
                    name: branch_name,
                };

                if let Some(company) = companies.last_mut() {
                    company.branches.push(branch);
                }
            }
        }

        companies
    }
}

fn make_sql_statment(table_name: &str, field_name: &str, uuid: &String, value: &String) -> String {
    format!(
        "INSERT OR IGNORE INTO {table_name} (rowid) VALUES ('{uuid}');
         UPDATE {table_name} SET {field_name} = '{value}' WHERE rowid = '{uuid}';"
    )
}
