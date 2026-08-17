use crate::utility::db_client;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::accounting_stuff;
use my_core::accounting_domain::utility::types;
use my_core::utility::traits;
use my_core::utility::utils::LogError;
use std::str::FromStr;
use uuid::Uuid;

pub struct S;

impl cases::get_all_accounts_for_branch::DatabaseRead for S {
    type Db<'a> = db_client::S;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::get_all_accounts_for_branch::ReadInput,
    ) -> Result<cases::get_all_accounts_for_branch::ReadOutput, traits::DynamicError> {
        // 1. Get the company UUID that owns this branch
        let branch_query = "
            SELECT company_belong FROM accounting_app.company_branch
            WHERE rowid = $1
        ";
        let branch_stmt = db.client.prepare_cached(branch_query).await.log()?;
        let branch_row = db
            .client
            .query_opt(&branch_stmt, &[&read_input.company_branch_uuid.to_externel_uuid()])
            .await
            .log()?;

        let company_uuid = match branch_row {
            Some(row) => {
                let company_uuid: Uuid = row.try_get(0).log()?;
                types::UuidType(company_uuid.into_bytes())
            }
            None => {
                return Err("Branch not found".into());
            }
        };

        // 2. Get all accounts that belong to that company
        let accounts_query = "
            SELECT
                rowid::text,
                is_debit,
                is_permanent_account,
                name,
                notes,
                unit_of_measurement_of_quantity
            FROM accounting_app.account
            WHERE belong_to_company = $1
        ";
        let accounts_stmt = db.client.prepare_cached(accounts_query).await.log()?;
        let account_rows =
            db.client.query(&accounts_stmt, &[&company_uuid.to_externel_uuid()]).await.log()?;

        let mut accounts = Vec::with_capacity(account_rows.len());
        for row in account_rows {
            let row_uuid_str: String = row.try_get(0).log()?;
            let row_uuid_parsed = Uuid::parse_str(&row_uuid_str).log()?;
            let row_uuid = types::UuidType(row_uuid_parsed.into_bytes());

            let is_debit: bool = row.try_get(1).log()?;
            let is_permanent_account: bool = row.try_get(2).log()?;
            let account_name: String = row.try_get(3).log()?;
            let notes: Option<String> = row.try_get(4).log()?;
            let unit_of_measurement_of_quantity: String = row.try_get(5).log()?;

            accounts.push(cases::get_all_accounts_for_branch::Account {
                row_uuid,
                is_debit,
                is_permanent_account,
                account_name,
                notes,
                unit_of_measurement_of_quantity,
            });
        }

        // 3. Get all account_flow_type entries for this branch
        let flow_query = "
            SELECT
                rowid::text,
                account,
                outflow_type,
                inflow_type
            FROM accounting_app.account_flow_type
            WHERE company_branch = $1
        ";
        let flow_stmt = db.client.prepare_cached(flow_query).await.log()?;
        let flow_rows = db
            .client
            .query(&flow_stmt, &[&read_input.company_branch_uuid.to_externel_uuid()])
            .await
            .log()?;

        let mut accounts_for_branch = Vec::with_capacity(flow_rows.len());
        for row in flow_rows {
            let row_uuid_str: String = row.try_get(0).log()?;
            let row_uuid_parsed = Uuid::parse_str(&row_uuid_str).log()?;
            let row_uuid = types::UuidType(row_uuid_parsed.into_bytes());

            let account_uuid_parsed: Uuid = row.try_get(1).log()?;
            let account_uuid = types::UuidType(account_uuid_parsed.into_bytes());

            let outflow_type_str: String = row.try_get(2).log()?;
            let outflow_type = accounting_stuff::OutFlowType::from_str(&outflow_type_str).log()?;

            let inflow_type_str: String = row.try_get(3).log()?;
            let inflow_type = accounting_stuff::InFlowType::from_str(&inflow_type_str).log()?;

            accounts_for_branch.push(cases::get_all_accounts_for_branch::AccountForBranch {
                row_uuid,
                account_uuid,
                outflow_type,
                inflow_type,
            });
        }

        Ok(cases::get_all_accounts_for_branch::ReadOutput {
            company_uuid,
            accounts,
            accounts_for_branch,
        })
    }
}
