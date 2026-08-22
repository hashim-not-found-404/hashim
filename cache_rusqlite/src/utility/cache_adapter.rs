use crate::utility::utils::MyUuidConverter;
use adapters::encode_decode;
use my_core::accounting_client::client_domain::cache::Cache;
use my_core::accounting_domain::request_response;
use my_core::accounting_domain::utility::resource_utils;
use my_core::accounting_domain::utility::types::JsonWebTokenType;
use my_core::accounting_domain::utility::types::UuidType;
use my_core::utility::traits::Coding;
use rusqlite::Connection;
use std::ops::Add;

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
        const TABLES_SCHEMA: &str = include_str!("../../schema/tables.sql");
        tables_db.execute_batch(TABLES_SCHEMA).unwrap();

        let transactions_db = Connection::open("opfs-sahpool://transactions.db").unwrap();
        const TRANSACTIONS_SCHEMA: &str = include_str!("../../schema/transactions.sql");
        transactions_db.execute_batch(TRANSACTIONS_SCHEMA).unwrap();

        Self {
            tables_db,
            transactions_db,
        }
    }

    async fn get_all_txn_input(
        &self,
    ) -> Vec<request_response::Txn<request_response::OperationsInput>> {
        let mut stmt = self.transactions_db.prepare(QUERY1).unwrap();

        let rows = stmt
            .query_map([], |row| {
                Ok(request_response::Txn {
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
        txn: &request_response::Txn<request_response::OperationsInput>,
    ) -> () {
        let txn_data = encode_decode::target::S::encode(&txn.operation);
        self.transactions_db
            .execute(QUERY2, rusqlite::params![txn.txn_number as i64, txn_data])
            .unwrap();
    }

    async fn write_txn_result(
        &self,
        txn: &request_response::Txn<request_response::OperationsResult>,
    ) {
        let txn_data = encode_decode::target::S::encode(&txn.operation);
        self.transactions_db
            .execute(QUERY3, rusqlite::params![txn.txn_number as i64, txn_data])
            .unwrap();
    }

    async fn mark_txn_input_as_faild(&self, txn_number: &u64) {
        self.transactions_db.execute(QUERY4, rusqlite::params![*txn_number as i64]).unwrap();
    }

    async fn delete_txn_input(&self, txn_number: &u64) {
        self.transactions_db.execute(QUERY5, rusqlite::params![*txn_number as i64]).unwrap();
    }

    async fn write_resource_from_server(&self, resource: &[resource_utils::ResourceInfo]) {
        write_resource(self, resource);
    }

    async fn write_resource_of_pending_txn(&self, resource: &[resource_utils::ResourceInfo]) {
        write_resource(self, resource);
    }

    async fn clear_pending_txn_state(&self) {
        self.tables_db.execute_batch(QUERY7).unwrap();
    }

    async fn start_pending_txn_state(&self) {
        self.tables_db.execute_batch(QUERY8).unwrap();
    }

    async fn get_jwt(&self, user_uuid: &UuidType) -> Option<JsonWebTokenType> {
        let mut stmt = self.tables_db.prepare(QUERY6).unwrap();

        let json_web_token_type = stmt.query_one([&user_uuid.to_string()], |row| row.get(0));

        match json_web_token_type {
            Ok(a) => Some(JsonWebTokenType(a)),
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

fn write_resource(db: &S, resource: &[resource_utils::ResourceInfo]) {
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
            resource_utils::Resource::TableAccessControlForCompanyBranchFieldDataGroup(value) => {
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
                make_sql_statment_for_bool("account", "is_debit", uuid, *value)
            }
            resource_utils::Resource::TableAccountFieldIsPermanentAccount(value) => {
                make_sql_statment_for_bool("account", "is_permanent_account", uuid, *value)
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
                make_sql_statment_for_string("entry", "shared_entry_id", uuid, &value.to_string())
            }
            resource_utils::Resource::TableSingleEntryFieldDoubleEntry(value) => {
                make_sql_statment_for_number("single_entry", "double_entry", uuid, &(*value as f64))
            }
            resource_utils::Resource::TableSingleEntryFieldEntry(value) => {
                make_sql_statment_for_string("single_entry", "entry", uuid, &value.to_string())
            }
            resource_utils::Resource::TableSingleEntryFieldAccount(value) => {
                make_sql_statment_for_string("single_entry", "account", uuid, &value.to_string())
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
    db.tables_db.execute_batch(stmts.as_str()).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::test_helper::test_query_helper_for_tables_schema;
    use crate::utility::test_helper::test_query_helper_for_transactions_schema;

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
