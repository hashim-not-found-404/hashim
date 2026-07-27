use crate::utility::cache_adapter;
use crate::utility::utils::MyUuidConverter;
use crate::utility::utils::MyUuidConverter1;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::accounting_stuff;
use my_core::utility::traits;
use rusqlite::params;
use std::str::FromStr;

pub struct S;

impl cases::get_all_accounts_for_branch::DatabaseRead for S {
    type Db<'a> = cache_adapter::S;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::get_all_accounts_for_branch::ReadInput,
    ) -> Result<cases::get_all_accounts_for_branch::ReadOutput, traits::DynamicError> {
        let branch_uuid_str = read_input.company_branch_uuid.to_string();

        // 1. Get the company UUID for this branch
        let mut stmt =
            db.db.prepare("SELECT company_belong FROM company_branch WHERE rowid = ?1").unwrap();
        let company_uuid_str: Option<String> =
            stmt.query_row(params![branch_uuid_str], |row| row.get::<_, String>(0)).ok();
        let company_uuid = match company_uuid_str {
            Some(s) => s.to_uuid(),
            None => {
                return Ok(cases::get_all_accounts_for_branch::ReadOutput::default());
            }
        };

        // 2. Get all accounts for that company
        let mut stmt = db.db.prepare(
            "SELECT rowid, is_debit, is_permanent_account, name, notes, unit_of_measurement_of_quantity
             FROM account WHERE belong_to_company = ?1"
        ).unwrap();
        let account_rows = stmt
            .query_map(params![company_uuid.to_string()], |row| {
                let row_uuid_str: String = row.get(0).unwrap();
                let is_debit: bool = row.get(1).unwrap();
                let is_permanent_account: bool = row.get(2).unwrap();
                let account_name: String = row.get(3).unwrap();
                let notes: String = row.get(4).unwrap();
                let unit_of_measurement: String = row.get(5).unwrap();
                Ok(cases::get_all_accounts_for_branch::Account {
                    row_uuid: row_uuid_str.to_uuid(),
                    is_debit,
                    is_permanent_account,
                    account_name,
                    notes,
                    unit_of_measurement_of_quantity: unit_of_measurement,
                })
            })
            .unwrap();

        let accounts = account_rows.collect::<Result<Vec<_>, _>>().unwrap();

        // 3. Get account_flow_type entries for this branch
        let mut stmt = db
            .db
            .prepare(
                "SELECT rowid, account, outflow_type, inflow_type
             FROM account_flow_type WHERE company_branch = ?1",
            )
            .unwrap();
        let flow_rows = stmt
            .query_map(params![branch_uuid_str], |row| {
                let row_uuid_str: String = row.get(0).unwrap();
                let account_uuid_str: String = row.get(1).unwrap();
                let outflow_type_str: String = row.get(2).unwrap();
                let inflow_type_str: String = row.get(3).unwrap();
                let outflow_type =
                    accounting_stuff::OutFlowType::from_str(&outflow_type_str).unwrap();
                let inflow_type = accounting_stuff::InFlowType::from_str(&inflow_type_str).unwrap();
                Ok(cases::get_all_accounts_for_branch::AccountForBranch {
                    row_uuid: row_uuid_str.to_uuid(),
                    account_uuid: account_uuid_str.to_uuid(),
                    outflow_type,
                    inflow_type,
                })
            })
            .unwrap();

        let accounts_for_branch = flow_rows.collect::<Result<Vec<_>, _>>().unwrap();

        Ok(cases::get_all_accounts_for_branch::ReadOutput {
            company_uuid,
            accounts,
            accounts_for_branch,
        })
    }
}
