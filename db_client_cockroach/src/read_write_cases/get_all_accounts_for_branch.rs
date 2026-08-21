use crate::utility::db_client;
use crate::utility::utils::MyUuidConverter;
use accounting_engine::accounting_stuff;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types::DatabaseRead;
use my_core::accounting_domain::utility::types::{self};
use my_core::utility::traits;
use my_core::utility::utils::LogError;
use std::str::FromStr;
use uuid::Uuid;

const QUERY1: &str = "
    SELECT company_belong FROM accounting_app.company_branch
    WHERE rowid = $1
";

const QUERY2: &str = "
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

const QUERY3: &str = "
    SELECT
        rowid::text,
        account,
        outflow_type,
        inflow_type
    FROM accounting_app.account_flow_type
    WHERE company_branch = $1
";

pub struct S;

impl cases::get_all_accounts_for_branch::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = db_client::S;
    type Error = traits::DynamicError;
    type ReadInput = cases::get_all_accounts_for_branch::ReadInput;
    type ReadOutput = cases::get_all_accounts_for_branch::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &Self::ReadInput,
    ) -> Result<Self::ReadOutput, Self::Error> {
        let branch_stmt = db.client.prepare_cached(QUERY1).await.log()?;
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

        let accounts_stmt = db.client.prepare_cached(QUERY2).await.log()?;
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

        let flow_stmt = db.client.prepare_cached(QUERY3).await.log()?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::test_helper::test_query_helper;

    #[tokio::test]
    async fn test_query_string_directly() {
        test_query_helper(QUERY1).await.unwrap();
        test_query_helper(QUERY2).await.unwrap();
        test_query_helper(QUERY3).await.unwrap();
    }
}
