use crate::prelude::*;
use rusqlite::{Connection, OptionalExtension, params};
use std::{ops::Add, str::FromStr};
use uuid::Uuid;

pub struct S {
    db: Connection,
}

impl Cache for S {
    async fn new() -> Self {
        let conn = Connection::open("opfs-sahpool://my_persistent_database.db").unwrap();

        const SCHEMA: &str = include_str!("../schema/tables.sql");
        conn.execute_batch(SCHEMA).unwrap();

        Self { db: conn }
    }

    async fn get_all_txn_input(&self) -> Vec<push_data::Txn<push_data::OperationsInput>> {
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
            let uuid = &reso.row_uuid.into_inner();

            let stmt = match &reso.resource {
                server_methods::Resource::Jwt(value) => {
                    make_sql_statment_for_string("user", "jwt", uuid, &value.0)
                }
                server_methods::Resource::HashedPassword(_) => continue,
                server_methods::Resource::TableUserFieldName(value) => {
                    make_sql_statment_for_string("user", "name", uuid, value)
                }
                server_methods::Resource::TableUserFieldId(value) => {
                    make_sql_statment_for_string("user", "id", uuid, value)
                }
                server_methods::Resource::TableCompanyFieldName(value) => {
                    make_sql_statment_for_string("company", "name", uuid, value)
                }
                server_methods::Resource::TableCompanyBranchFieldName(value) => {
                    make_sql_statment_for_string("company_branch", "name", uuid, value)
                }
                server_methods::Resource::TableCompanyBranchFieldCompanyBelong(value) => {
                    make_sql_statment_for_string(
                        "company_branch",
                        "company_belong",
                        uuid,
                        &value.into_inner(),
                    )
                }
                server_methods::Resource::TableCompanyBranchFieldCurrency(value) => {
                    make_sql_statment_for_string(
                        "company_branch",
                        "currency",
                        uuid,
                        &value.as_str().to_string(),
                    )
                }
                server_methods::Resource::TableCompanyBranchFieldLocation(value) => {
                    make_sql_statment_for_number(
                        "company_branch",
                        "location_latitude",
                        uuid,
                        &value.latitude,
                    )
                    .add(
                        make_sql_statment_for_number(
                            "company_branch",
                            "location_longitude",
                            uuid,
                            &value.longitude,
                        )
                        .as_str(),
                    )
                }
                server_methods::Resource::TableCompanyFieldCurrency(value) => {
                    make_sql_statment_for_string(
                        "company",
                        "currency",
                        uuid,
                        &value.as_str().to_string(),
                    )
                }
                server_methods::Resource::TableAccessControlForCompanyFieldRole(value) => {
                    make_sql_statment_for_string(
                        "access_control_for_company",
                        "role",
                        uuid,
                        &value.as_str().to_string(),
                    )
                }
                server_methods::Resource::TableAccessControlForCompanyFieldUser(value) => {
                    make_sql_statment_for_string(
                        "access_control_for_company",
                        "user_",
                        uuid,
                        &value.into_inner(),
                    )
                }
                server_methods::Resource::TableAccessControlForCompanyFieldDataGroup(value) => {
                    make_sql_statment_for_string(
                        "access_control_for_company",
                        "data_group",
                        uuid,
                        &value.into_inner(),
                    )
                }
                server_methods::Resource::TableAccessControlForCompanyBranchFieldRole(value) => {
                    make_sql_statment_for_string(
                        "access_control_for_company_branch",
                        "role",
                        uuid,
                        &value.as_str().to_string(),
                    )
                }
                server_methods::Resource::TableAccessControlForCompanyBranchFieldUser(value) => {
                    make_sql_statment_for_string(
                        "access_control_for_company_branch",
                        "user_",
                        uuid,
                        &value.into_inner(),
                    )
                }
                server_methods::Resource::TableAccessControlForCompanyBranchFieldDataGroup(
                    value,
                ) => make_sql_statment_for_string(
                    "access_control_for_company_branch",
                    "data_group",
                    uuid,
                    &value.into_inner(),
                ),
            };

            stmts.push(stmt);
        }

        let stmts = stmts.concat();
        self.db.execute_batch(stmts.as_str()).unwrap();
    }

    async fn get_jwt(&self, user_uuid: &db_types::UuidType) -> Option<db_types::JsonWebTokenType> {
        let mut stmt = self
            .db
            .prepare("SELECT jwt FROM user WHERE rowid = ?1")
            .unwrap();

        let json_web_token_type = stmt.query_one([&user_uuid.0], |row| row.get(0));

        match json_web_token_type {
            Ok(a) => Some(db_types::JsonWebTokenType(a)),
            Err(_) => None,
        }
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
                    user_uuid_str.to_uuid(),
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
    ) -> Vec<ResourceInfo> {
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

        let mut resources = Vec::new();
        let mut last_company_uuid: Option<String> = None;

        for row in rows {
            let (company_uuid_str, company_name, user_role_str, branch_uuid_opt, branch_name_opt) =
                row.unwrap();

            // If this is a new company, add company-level resources once
            if last_company_uuid.as_ref() != Some(&company_uuid_str) {
                last_company_uuid = Some(company_uuid_str.clone());

                let company_uuid_db = company_uuid_str.clone().to_uuid();
                let role = db_types::Role::from_str(&user_role_str).unwrap();

                // Company name
                resources.push(ResourceInfo {
                    row_uuid: company_uuid_db.clone(),
                    resource: server_methods::Resource::TableCompanyFieldName(company_name),
                });

                // Access control: role
                resources.push(ResourceInfo {
                    row_uuid: company_uuid_db.clone(),
                    resource: server_methods::Resource::TableAccessControlForCompanyFieldRole(role),
                });

                // Access control: user
                resources.push(ResourceInfo {
                    row_uuid: company_uuid_db.clone(),
                    resource: server_methods::Resource::TableAccessControlForCompanyFieldUser(
                        user_uuid.clone(),
                    ),
                });

                // Access control: data_group (self)
                resources.push(ResourceInfo {
                    row_uuid: company_uuid_db.clone(),
                    resource: server_methods::Resource::TableAccessControlForCompanyFieldDataGroup(
                        company_uuid_db.clone(),
                    ),
                });
            }

            // Add branch resources if branch exists
            if let (Some(branch_uuid_str), Some(branch_name)) = (branch_uuid_opt, branch_name_opt) {
                let branch_uuid_db = branch_uuid_str.to_uuid();
                let company_uuid_db = company_uuid_str.to_uuid();

                resources.push(ResourceInfo {
                    row_uuid: branch_uuid_db.clone(),
                    resource: server_methods::Resource::TableCompanyBranchFieldName(branch_name),
                });
                resources.push(ResourceInfo {
                    row_uuid: branch_uuid_db,
                    resource: server_methods::Resource::TableCompanyBranchFieldCompanyBelong(
                        company_uuid_db,
                    ),
                });
            }
        }

        resources
    }

    async fn read_create_company_branch(
        &self,
        user_uuid: &db_types::UuidType,
        company_belong: &db_types::UuidType,
        company_branch_name: &String,
    ) -> (
        Vec<db_types::Role>, /* roles at company */
        bool,                /* is company exist */
        bool,                /* is branch name used */
    ) {
        // 1. Get the user's roles in the company
        let mut stmt = self
            .db
            .prepare(
                "SELECT role FROM access_control_for_company WHERE data_group = ?1 AND user_ = ?2",
            )
            .unwrap();

        let roles_iter = stmt
            .query_map(params![company_belong.0, user_uuid.0], |row| {
                let role_str: String = row.get(0)?;
                let role = db_types::Role::from_str(role_str.as_str()).unwrap();
                Ok(role)
            })
            .unwrap();

        let mut roles = Vec::new();
        for role in roles_iter {
            roles.push(role.unwrap());
        }

        // 2. Check if the company exists
        let mut stmt = self
            .db
            .prepare("SELECT 1 FROM company WHERE rowid = ?1")
            .unwrap();
        let company_exists = stmt.exists(params![company_belong.0]).unwrap();

        // 3. Check if the branch name is already used under this company
        let mut stmt = self
            .db
            .prepare("SELECT 1 FROM company_branch WHERE company_belong = ?1 AND name = ?2")
            .unwrap();
        let branch_name_used = stmt
            .exists(params![company_belong.0, company_branch_name])
            .unwrap();

        (roles, company_exists, branch_name_used)
    }
}

fn make_sql_statment_for_string(
    table_name: &str,
    field_name: &str,
    uuid: &String,
    value: &String,
) -> String {
    format!(
        "INSERT OR IGNORE INTO {table_name} (rowid) VALUES ('{uuid}');
         UPDATE {table_name} SET {field_name} = '{value}' WHERE rowid = '{uuid}';"
    )
}

fn make_sql_statment_for_number(
    table_name: &str,
    field_name: &str,
    uuid: &String,
    value: &f64,
) -> String {
    format!(
        "INSERT OR IGNORE INTO {table_name} (rowid) VALUES ('{uuid}');
         UPDATE {table_name} SET {field_name} = {value} WHERE rowid = '{uuid}';"
    )
}

pub trait MyUuidConverter {
    fn into_inner(&self) -> String;
}

impl MyUuidConverter for db_types::UuidType {
    fn into_inner(&self) -> String {
        // Convert [u8; 16] → Uuid → String
        let uuid = Uuid::from_bytes(self.0);
        uuid.to_string()
    }
}

pub trait MyUuidConverter1 {
    fn to_uuid(self) -> db_types::UuidType;
}

impl MyUuidConverter1 for String {
    fn to_uuid(self) -> db_types::UuidType {
        // Parse string → Uuid → [u8; 16]
        let uuid = Uuid::parse_str(&self).expect("Invalid UUID string");
        db_types::UuidType(*uuid.as_bytes())
    }
}
