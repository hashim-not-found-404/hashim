use crate::utils::{MyUuidConverter, MyUuidConverter1};
use adapters::encode_decode;
use my_core::{
    accounting_client::use_cases::client_domain::cache::Cache,
    accounting_domain::{
        cases::{self, utility::types},
        request_response,
    },
    utility::traits::Coding,
};
use rusqlite::{Connection, OptionalExtension, params};
use std::{ops::Add, str::FromStr};

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

    async fn get_all_txn_input(
        &self,
    ) -> Vec<request_response::push_data::Txn<request_response::push_data::OperationsInput>> {
        let mut stmt = self
            .db
            .prepare(
                "SELECT txn_number, txn FROM write_cache_transactions_input WHERE is_faild = false",
            )
            .unwrap();

        let rows = stmt
            .query_map([], |row| {
                Ok(request_response::push_data::Txn {
                    txn_number: row.get::<usize, i64>(0).unwrap() as u64,
                    operation: encode_decode::target::S::decode(
                        &row.get::<usize, Vec<u8>>(1).unwrap(),
                    )
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

    async fn write_txn_input(
        &self,
        txn: &request_response::push_data::Txn<request_response::push_data::OperationsInput>,
    ) -> () {
        let txn_data = encode_decode::target::S::encode(&txn.operation);
        self.db
            .execute(
                "INSERT OR REPLACE INTO write_cache_transactions_input (txn_number, txn) VALUES (?1, ?2)",
                rusqlite::params![txn.txn_number as i64, txn_data],
            )
            .unwrap();
    }

    async fn write_txn_result(
        &self,
        txn: &request_response::push_data::Txn<request_response::push_data::OperationsResult>,
    ) {
        let txn_data = encode_decode::target::S::encode(&txn.operation);
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

    async fn write_resource(&self, resource: &Vec<types::ResourceInfo>) {
        let mut stmts = Vec::with_capacity(resource.len());

        for reso in resource {
            let uuid = &reso.row_uuid.to_string();

            let stmt = match &reso.resource {
                types::Resource::Jwt(value) => {
                    make_sql_statment_for_string("user", "jwt", uuid, &value.0)
                }
                types::Resource::TableUserFieldName(value) => {
                    make_sql_statment_for_string("user", "name", uuid, value)
                }
                types::Resource::TableUserFieldId(value) => {
                    make_sql_statment_for_string("user", "id", uuid, value)
                }
                types::Resource::TableCompanyFieldName(value) => {
                    make_sql_statment_for_string("company", "name", uuid, value)
                }
                types::Resource::TableCompanyBranchFieldName(value) => {
                    make_sql_statment_for_string("company_branch", "name", uuid, value)
                }
                types::Resource::TableCompanyBranchFieldCompanyBelong(value) => {
                    make_sql_statment_for_string(
                        "company_branch",
                        "company_belong",
                        uuid,
                        &value.to_string(),
                    )
                }
                types::Resource::TableCompanyBranchFieldCurrency(value) => {
                    make_sql_statment_for_string(
                        "company_branch",
                        "currency",
                        uuid,
                        &value.as_str().to_string(),
                    )
                }
                types::Resource::TableCompanyBranchFieldLocation(value) => {
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
                types::Resource::TableCompanyFieldCurrency(value) => make_sql_statment_for_string(
                    "company",
                    "currency",
                    uuid,
                    &value.as_str().to_string(),
                ),
                types::Resource::TableAccessControlForCompanyFieldRole(value) => {
                    make_sql_statment_for_string(
                        "access_control_for_company",
                        "role",
                        uuid,
                        &value.as_str().to_string(),
                    )
                }
                types::Resource::TableAccessControlForCompanyFieldUser(value) => {
                    make_sql_statment_for_string(
                        "access_control_for_company",
                        "user_",
                        uuid,
                        &value.to_string(),
                    )
                }
                types::Resource::TableAccessControlForCompanyFieldDataGroup(value) => {
                    make_sql_statment_for_string(
                        "access_control_for_company",
                        "data_group",
                        uuid,
                        &value.to_string(),
                    )
                }
                types::Resource::TableAccessControlForCompanyBranchFieldRole(value) => {
                    make_sql_statment_for_string(
                        "access_control_for_company_branch",
                        "role",
                        uuid,
                        &value.as_str().to_string(),
                    )
                }
                types::Resource::TableAccessControlForCompanyBranchFieldUser(value) => {
                    make_sql_statment_for_string(
                        "access_control_for_company_branch",
                        "user_",
                        uuid,
                        &value.to_string(),
                    )
                }
                types::Resource::TableAccessControlForCompanyBranchFieldDataGroup(value) => {
                    make_sql_statment_for_string(
                        "access_control_for_company_branch",
                        "data_group",
                        uuid,
                        &value.to_string(),
                    )
                }
            };

            stmts.push(stmt);
        }

        let stmts = stmts.concat();
        self.db.execute_batch(stmts.as_str()).unwrap();
    }

    async fn get_jwt(&self, user_uuid: &types::UuidType) -> Option<types::JsonWebTokenType> {
        let mut stmt = self
            .db
            .prepare("SELECT jwt FROM user WHERE rowid = ?1")
            .unwrap();

        let json_web_token_type = stmt.query_one([&user_uuid.to_string()], |row| row.get(0));

        match json_web_token_type {
            Ok(a) => Some(types::JsonWebTokenType(a)),
            Err(_) => None,
        }
    }

    async fn read_sign_up(
        &self,
        new_uuid: &types::UuidType,
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
            .query_one(query, params![new_uuid.to_string(), user_id], |row| {
                Ok((row.get(0).unwrap(), row.get(1).unwrap()))
            })
            .unwrap()
    }

    async fn read_sign_in(
        &self,
        user_id: &String,
    ) -> Option<(
        types::UuidType, /* user uuid */
        Option<String>,  /* user name */
        bool,            /* does he have jwt */
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
        user_uuid: &types::UuidType,
    ) -> Vec<cases::list_company_and_branch::AllCompaniesThatUserInWithRoles> {
        use std::collections::HashMap;
        use types::Role;

        // ---- 1. Get company-level roles ----
        let company_query = "
            SELECT c.rowid, c.name, c.currency, acf.role
            FROM access_control_for_company acf
            JOIN company c ON acf.data_group = c.rowid
            WHERE acf.user_ = ?1
        ";
        let mut stmt = self.db.prepare(company_query).unwrap();
        let company_rows = stmt
            .query_map(params![user_uuid.to_string()], |row| {
                let uuid: String = row.get(0).unwrap();
                let name: String = row.get(1).unwrap();
                let currency: String = row.get(2).unwrap();
                let role: Option<String> = row.get(3).unwrap(); // read as Option<String>
                Ok((uuid, name, currency, role))
            })
            .unwrap();

        // Group company roles by company UUID
        let mut company_map: HashMap<String, (String, String, Vec<Role>)> = HashMap::new();
        for row in company_rows {
            let (uuid, name, currency, role_opt) = row.unwrap();
            if let Some(role_str) = role_opt {
                let role = Role::from_str(&role_str).unwrap();
                company_map
                    .entry(uuid)
                    .or_insert_with(|| (name, currency, Vec::new()))
                    .2
                    .push(role);
            }
            // If role is None, skip? Or still create company entry with empty roles?
            // The code above creates the entry only when a role exists.
            // To include companies with no roles, you'd need to create the entry regardless.
            // But since we only get rows for companies with at least one role (due to the JOIN), it's fine.
        }

        // ---- 2. Get branch-level roles ----
        let branch_query = "
            SELECT cb.rowid, cb.name, cb.currency, cb.company_belong, acfb.role
            FROM access_control_for_company_branch acfb
            JOIN company_branch cb ON acfb.data_group = cb.rowid
            WHERE acfb.user_ = ?1
        ";
        let mut stmt = self.db.prepare(branch_query).unwrap();
        let branch_rows = stmt
            .query_map(params![user_uuid.to_string()], |row| {
                let branch_uuid: String = row.get(0).unwrap();
                let branch_name: String = row.get(1).unwrap();
                let branch_currency: String = row.get(2).unwrap();
                let company_belong: String = row.get(3).unwrap();
                let role: Option<String> = row.get(4).unwrap(); // read as Option<String>
                Ok((
                    branch_uuid,
                    branch_name,
                    branch_currency,
                    company_belong,
                    role,
                ))
            })
            .unwrap();

        // Group branch roles by branch UUID, and remember which company it belongs to
        struct BranchAccumulator {
            branch_uuid: String,
            branch_name: String,
            branch_currency: String,
            company_belong: String,
            roles: Vec<Role>,
        }
        let mut branch_map: HashMap<String, BranchAccumulator> = HashMap::new();
        for row in branch_rows {
            let (branch_uuid, branch_name, branch_currency, company_belong, role_opt) =
                row.unwrap();
            // Create entry for the branch (even if role is None)
            let entry =
                branch_map
                    .entry(branch_uuid.clone())
                    .or_insert_with(|| BranchAccumulator {
                        branch_uuid: branch_uuid.clone(),
                        branch_name: branch_name.clone(),
                        branch_currency: branch_currency.clone(),
                        company_belong: company_belong.clone(),
                        roles: Vec::new(),
                    });
            if let Some(role_str) = role_opt {
                let role = Role::from_str(&role_str).unwrap();
                entry.roles.push(role);
            }
            // If role is None, we still keep the branch entry with empty roles.
        }

        // ---- 3. Build the final result ----
        let mut result = Vec::new();
        for (company_uuid_str, (company_name, company_currency_str, company_roles)) in company_map {
            let company_uuid = company_uuid_str.clone().to_uuid();
            let company_currency = types::Currency::from_str(&company_currency_str).unwrap();

            // Collect branches that belong to this company
            let branches: Vec<cases::list_company_and_branch::AllBranchesThatUserInWithRoles> =
                branch_map
                    .iter()
                    .filter(|(_, info)| info.company_belong == company_uuid_str)
                    .map(|(_, info)| {
                        let branch_uuid = info.branch_uuid.clone().to_uuid();
                        let branch_currency =
                            types::Currency::from_str(&info.branch_currency).unwrap();
                        cases::list_company_and_branch::AllBranchesThatUserInWithRoles {
                            branch_uuid,
                            branch_name: info.branch_name.clone(),
                            branch_currancy: branch_currency,
                            user_roles: info.roles.clone(),
                        }
                    })
                    .collect::<Vec<_>>();

            result.push(
                cases::list_company_and_branch::AllCompaniesThatUserInWithRoles {
                    company_uuid,
                    company_name,
                    company_currancy: company_currency,
                    user_roles: company_roles,
                    branches,
                },
            );
        }

        result
    }

    async fn read_create_company_branch(
        &self,
        user_uuid: &types::UuidType,
        company_belong: &types::UuidType,
        company_branch_name: &String,
    ) -> (
        Vec<types::Role>, /* roles at company */
        bool,             /* is company exist */
        bool,             /* is branch name used */
    ) {
        // 1. Get the user's roles in the company
        let mut stmt = self
            .db
            .prepare(
                "SELECT role FROM access_control_for_company WHERE data_group = ?1 AND user_ = ?2",
            )
            .unwrap();

        let roles_iter = stmt
            .query_map(
                params![company_belong.to_string(), user_uuid.to_string()],
                |row| {
                    let role_str: String = row.get(0)?;
                    let role = types::Role::from_str(role_str.as_str()).unwrap();
                    Ok(role)
                },
            )
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
        let company_exists = stmt.exists(params![company_belong.to_string()]).unwrap();

        // 3. Check if the branch name is already used under this company
        let mut stmt = self
            .db
            .prepare("SELECT 1 FROM company_branch WHERE company_belong = ?1 AND name = ?2")
            .unwrap();
        let branch_name_used = stmt
            .exists(params![company_belong.to_string(), company_branch_name])
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
