use crate::utility::utils::MyUuidConverter;
use adapters::encode_decode;
use my_core::accounting_client::client_domain::cache::Cache;
use my_core::accounting_domain::request_response;
use my_core::accounting_domain::utility::resource_utils;
use my_core::accounting_domain::utility::types;
use my_core::utility::traits::Coding;
use rusqlite::Connection;
use std::ops::Add;

pub struct S {
    pub(crate) db: Connection,
}

impl Cache for S {
    async fn new() -> Self {
        let conn = Connection::open("opfs-sahpool://my_persistent_database.db").unwrap();

        const SCHEMA: &str = include_str!("../../schema/tables.sql");
        conn.execute_batch(SCHEMA).unwrap();

        Self {
            db: conn,
        }
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
                    operation:  encode_decode::target::S::decode(
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

    async fn write_resource(&self, resource: &Vec<resource_utils::ResourceInfo>) {
        let mut stmts = Vec::with_capacity(resource.len());

        for reso in resource {
            let uuid = &reso.row_uuid.to_string();

            let stmt = match &reso.resource {
                resource_utils::Resource::Jwt(value) => {
                    make_sql_statment_for_string("user", "jwt", uuid, &value.0)
                }
                resource_utils::Resource::TableUserFieldName(value) => {
                    make_sql_statment_for_string("user", "name", uuid, value)
                }
                resource_utils::Resource::TableUserFieldId(value) => {
                    make_sql_statment_for_string("user", "id", uuid, value)
                }
                resource_utils::Resource::TableCompanyFieldName(value) => {
                    make_sql_statment_for_string("company", "name", uuid, value)
                }
                resource_utils::Resource::TableCompanyBranchFieldName(value) => {
                    make_sql_statment_for_string("company_branch", "name", uuid, value)
                }
                resource_utils::Resource::TableCompanyBranchFieldCompanyBelong(value) => {
                    make_sql_statment_for_string(
                        "company_branch",
                        "company_belong",
                        uuid,
                        &value.to_string(),
                    )
                }
                resource_utils::Resource::TableCompanyBranchFieldCurrency(value) => {
                    make_sql_statment_for_string(
                        "company_branch",
                        "currency",
                        uuid,
                        &value.as_str().to_string(),
                    )
                }
                resource_utils::Resource::TableCompanyBranchFieldLocation(value) => {
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
                resource_utils::Resource::TableCompanyFieldCurrency(value) => {
                    make_sql_statment_for_string(
                        "company",
                        "currency",
                        uuid,
                        &value.as_str().to_string(),
                    )
                }
                resource_utils::Resource::TableAccessControlForCompanyFieldRole(value) => {
                    make_sql_statment_for_string(
                        "access_control_for_company",
                        "role",
                        uuid,
                        &value.as_str().to_string(),
                    )
                }
                resource_utils::Resource::TableAccessControlForCompanyFieldUser(value) => {
                    make_sql_statment_for_string(
                        "access_control_for_company",
                        "user_",
                        uuid,
                        &value.to_string(),
                    )
                }
                resource_utils::Resource::TableAccessControlForCompanyFieldDataGroup(value) => {
                    make_sql_statment_for_string(
                        "access_control_for_company",
                        "data_group",
                        uuid,
                        &value.to_string(),
                    )
                }
                resource_utils::Resource::TableAccessControlForCompanyBranchFieldRole(value) => {
                    make_sql_statment_for_string(
                        "access_control_for_company_branch",
                        "role",
                        uuid,
                        &value.as_str().to_string(),
                    )
                }
                resource_utils::Resource::TableAccessControlForCompanyBranchFieldUser(value) => {
                    make_sql_statment_for_string(
                        "access_control_for_company_branch",
                        "user_",
                        uuid,
                        &value.to_string(),
                    )
                }
                resource_utils::Resource::TableAccessControlForCompanyBranchFieldDataGroup(
                    value,
                ) => {
                    make_sql_statment_for_string(
                        "access_control_for_company_branch",
                        "data_group",
                        uuid,
                        &value.to_string(),
                    )
                }
                resource_utils::Resource::TableAccountFieldCompanyBelong(value) => {
                    make_sql_statment_for_string(
                        "account",
                        "belong_to_company",
                        uuid,
                        &value.to_string(),
                    )
                }
                resource_utils::Resource::TableAccountFieldIsDebit(value) => {
                    make_sql_statment_for_bool("account", "is_debit", uuid, value.clone())
                }
                resource_utils::Resource::TableAccountFieldIsPermanentAccount(value) => {
                    make_sql_statment_for_bool(
                        "account",
                        "is_permanent_account",
                        uuid,
                        value.clone(),
                    )
                }
                resource_utils::Resource::TableAccountFieldName(value) => {
                    make_sql_statment_for_string("account", "name", uuid, value)
                }
                resource_utils::Resource::TableAccountFieldNotes(value) => {
                    make_sql_statement_for_option_string("account", "notes", uuid, value)
                }
                resource_utils::Resource::TableAccountFieldUnitOfMeasurementOfQuantity(value) => {
                    make_sql_statment_for_string(
                        "account",
                        "unit_of_measurement_of_quantity",
                        uuid,
                        value,
                    )
                }
                resource_utils::Resource::TableAccountFlowTypeFieldAccount(value) => {
                    make_sql_statment_for_string(
                        "account_flow_type",
                        "account",
                        uuid,
                        &value.to_string(),
                    )
                }
                resource_utils::Resource::TableAccountFlowTypeFieldCompanyBranch(value) => {
                    make_sql_statment_for_string(
                        "account_flow_type",
                        "company_branch",
                        uuid,
                        &value.to_string(),
                    )
                }
                resource_utils::Resource::TableAccountFlowTypeFieldInflowType(value) => {
                    make_sql_statment_for_string(
                        "account_flow_type",
                        "inflow_type",
                        uuid,
                        &value.as_str().to_string(),
                    )
                }
                resource_utils::Resource::TableAccountFlowTypeFieldOutflowType(value) => {
                    make_sql_statment_for_string(
                        "account_flow_type",
                        "outflow_type",
                        uuid,
                        &value.as_str().to_string(),
                    )
                }
                resource_utils::Resource::TableAccountFieldInventory(value) => {
                    let json = serde_json::to_string(value).unwrap();
                    make_sql_statment_for_string("account", "inventory", uuid, &json)
                }
                resource_utils::Resource::TableSharedEntryFieldWriter(value) => {
                    make_sql_statment_for_string("shared_entry", "writer", uuid, &value.to_string())
                }
                resource_utils::Resource::TableSharedEntryFieldNotes(value) => {
                    make_sql_statement_for_option_string("shared_entry", "notes", uuid, value)
                }
                resource_utils::Resource::TableEntryFieldWriter(value) => {
                    make_sql_statment_for_string("entry", "writer", uuid, &value.to_string())
                }
                resource_utils::Resource::TableEntryFieldTime(value) => {
                    make_sql_statment_for_number("entry", "time", uuid, &(*value as f64))
                }
                resource_utils::Resource::TableEntryFieldSharedEntryId(value) => {
                    make_sql_statment_for_string(
                        "entry",
                        "shared_entry_id",
                        uuid,
                        &value.to_string(),
                    )
                }
                resource_utils::Resource::TableSingleEntryFieldDoubleEntry(value) => {
                    make_sql_statment_for_number(
                        "single_entry",
                        "double_entry",
                        uuid,
                        &(*value as f64),
                    )
                }
                resource_utils::Resource::TableSingleEntryFieldEntry(value) => {
                    make_sql_statment_for_string("single_entry", "entry", uuid, &value.to_string())
                }
                resource_utils::Resource::TableSingleEntryFieldAccount(value) => {
                    make_sql_statment_for_string(
                        "single_entry",
                        "account",
                        uuid,
                        &value.to_string(),
                    )
                }
                resource_utils::Resource::TableSingleEntryFieldIsDebit(value) => {
                    make_sql_statment_for_bool("single_entry", "is_debit", uuid, *value)
                }
                resource_utils::Resource::TableSingleEntryFieldCostOutFlowType(value) => {
                    make_sql_statment_for_string(
                        "single_entry",
                        "cost_out_flow_type",
                        uuid,
                        &value.as_str().to_string(),
                    )
                }
                resource_utils::Resource::TableSingleEntryFieldQuantity(value) => {
                    make_sql_statment_for_number("single_entry", "quantity", uuid, value)
                }
                resource_utils::Resource::TableSingleEntryFieldAmount(value) => {
                    make_sql_statment_for_number("single_entry", "amount", uuid, value)
                }
            };

            stmts.push(stmt);
        }

        let stmts = stmts.concat();
        self.db.execute_batch(stmts.as_str()).unwrap();
    }

    async fn get_jwt(&self, user_uuid: &types::UuidType) -> Option<types::JsonWebTokenType> {
        let mut stmt = self.db.prepare("SELECT jwt FROM user WHERE rowid = ?1").unwrap();

        let json_web_token_type = stmt.query_one([&user_uuid.to_string()], |row| row.get(0));

        match json_web_token_type {
            Ok(a) => Some(types::JsonWebTokenType(a)),
            Err(_) => None,
        }
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

fn make_sql_statment_for_bool(
    table_name: &str,
    field_name: &str,
    uuid: &String,
    value: bool,
) -> String {
    format!(
        "INSERT OR IGNORE INTO {table_name} (rowid) VALUES ('{uuid}');
         UPDATE {table_name} SET {field_name} = {value} WHERE rowid = '{uuid}';"
    )
}

fn make_sql_statement_for_option_string(
    table_name: &str,
    field_name: &str,
    uuid: &String,
    value: &Option<String>,
) -> String {
    match value {
        Some(v) => make_sql_statment_for_string(table_name, field_name, uuid, v),
        None => {
            format!(
                "INSERT OR IGNORE INTO {table_name} (rowid) VALUES ('{uuid}');
             UPDATE {table_name} SET {field_name} = NULL WHERE rowid = '{uuid}';"
            )
        }
    }
}
