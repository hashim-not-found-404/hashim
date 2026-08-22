use crate::utility::cache_adapter;
use crate::utility::utils::MyUuidConverter;
use crate::utility::utils::MyUuidConverter1;
use accounting_engine::accounting_stuff;
use my_core::domain::use_cases;
use my_core::domain::utility::types::DatabaseRead;
use my_core::domain::utility::uuid::Company;
use my_core::utility::traits::DynamicError;
use rusqlite::params;
use std::str::FromStr;

const QUERY1: &str = "SELECT company_belong FROM company_branch WHERE rowid = ?1";
const QUERY2: &str =
    "SELECT rowid, is_debit, is_permanent_account, name, notes, unit_of_measurement_of_quantity
             FROM account WHERE belong_to_company = ?1";
const QUERY3: &str = "SELECT rowid, account, outflow_type, inflow_type
             FROM account_flow_type WHERE company_branch = ?1";

pub struct S;

impl use_cases::get_all_accounts_for_branch::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = cache_adapter::S;
    type Input = use_cases::get_all_accounts_for_branch::ReadInput;
    type Output = use_cases::get_all_accounts_for_branch::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        input: &Self::Input,
    ) -> Result<Self::Output, DynamicError> {
        let branch_uuid_str = input.company_branch_uuid.to_string();

        let mut stmt = db.tables_db.prepare(QUERY1).unwrap();
        let company_uuid_str: Option<String> =
            stmt.query_row(params![branch_uuid_str], |row| row.get::<_, String>(0)).ok();
        let company_uuid: Company = match company_uuid_str {
            Some(s) => s.to_uuid().into(),
            None => {
                return Ok(use_cases::get_all_accounts_for_branch::ReadOutput::default());
            }
        };

        let mut stmt = db.tables_db.prepare(QUERY2).unwrap();
        let account_rows = stmt
            .query_map(params![company_uuid.to_string()], |row| {
                let row_uuid_str: String = row.get(0)?;
                let is_debit: bool = row.get(1)?;
                let is_permanent_account: bool = row.get(2)?;
                let account_name: String = row.get(3)?;
                let notes: Option<String> = row.get(4)?;
                let unit_of_measurement: String = row.get(5)?;
                Ok(use_cases::get_all_accounts_for_branch::Account {
                    row_uuid: row_uuid_str.to_uuid().into(),
                    is_debit,
                    is_permanent_account,
                    account_name,
                    notes,
                    unit_of_measurement_of_quantity: unit_of_measurement,
                })
            })
            .unwrap();

        let accounts: Vec<use_cases::get_all_accounts_for_branch::Account> =
            account_rows.filter_map(|row| row.ok()).collect();

        let mut stmt = db.tables_db.prepare(QUERY3).unwrap();
        let flow_rows = stmt
            .query_map(params![branch_uuid_str], |row| {
                let row_uuid_str: String = row.get(0).unwrap();
                let account_uuid_str: String = row.get(1).unwrap();
                let outflow_type_str: String = row.get(2).unwrap();
                let inflow_type_str: String = row.get(3).unwrap();
                let outflow_type =
                    accounting_stuff::OutFlowType::from_str(&outflow_type_str).unwrap();
                let inflow_type = accounting_stuff::InFlowType::from_str(&inflow_type_str).unwrap();
                Ok(use_cases::get_all_accounts_for_branch::AccountForBranch {
                    row_uuid: row_uuid_str.to_uuid().into(),
                    account_uuid: account_uuid_str.to_uuid().into(),
                    outflow_type,
                    inflow_type,
                })
            })
            .unwrap();

        let accounts_for_branch = flow_rows.collect::<Result<Vec<_>, _>>().unwrap();

        Ok(use_cases::get_all_accounts_for_branch::ReadOutput {
            company_uuid,
            accounts,
            accounts_for_branch,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::test_helper::test_query_helper_for_tables_schema;

    #[test]
    fn test_query_string_directly() {
        test_query_helper_for_tables_schema(QUERY1).unwrap();
        test_query_helper_for_tables_schema(QUERY2).unwrap();
        test_query_helper_for_tables_schema(QUERY3).unwrap();
    }
}
