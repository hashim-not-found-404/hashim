use crate::utility::db_client;
use crate::utility::utils::MyUuidConverter;
use accounting_engine::accounting_stuff;
use my_core::domain::use_cases;
use my_core::domain::utility::new_types::UuidType;
use my_core::domain::utility::types::DatabaseRead;
use my_core::utility::traits::DynamicError;
use my_core::utility::utils::LogError;
use serde_json::Value;
use std::str::FromStr;
use uuid::Uuid;

const READ_QUERY: &str = r#"
    WITH branch AS (
        SELECT company_belong
        FROM accounting_app.company_branch
        WHERE rowid = $1
    ),
    accounts AS (
        SELECT COALESCE(
            jsonb_agg(
                jsonb_build_object(
                    'rowid', rowid::text,
                    'is_debit', is_debit,
                    'is_permanent_account', is_permanent_account,
                    'name', name,
                    'notes', notes,
                    'unit_of_measurement_of_quantity', unit_of_measurement_of_quantity
                )
                ORDER BY name
            ),
            '[]'::jsonb
        ) AS data
        FROM accounting_app.account
        WHERE belong_to_company = (SELECT company_belong FROM branch)
    ),
    flows AS (
        SELECT COALESCE(
            jsonb_agg(
                jsonb_build_object(
                    'rowid', rowid::text,
                    'account', account::text,
                    'outflow_type', outflow_type,
                    'inflow_type', inflow_type
                )
            ),
            '[]'::jsonb
        ) AS data
        FROM accounting_app.account_flow_type
        WHERE company_branch = $1
    )
    SELECT
        (SELECT company_belong FROM branch) AS company_uuid,
        (SELECT data FROM accounts) AS accounts_json,
        (SELECT data FROM flows) AS flows_json
"#;

pub struct S;

impl use_cases::get_all_accounts_for_branch::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = db_client::S;
    type Input = use_cases::get_all_accounts_for_branch::ReadInput;
    type Output = use_cases::get_all_accounts_for_branch::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        input: &Self::Input,
    ) -> Result<Self::Output, DynamicError> {
        let stmt = db.client.prepare_cached(READ_QUERY).await.log()?;
        let row = db
            .client
            .query_one(&stmt, &[&input.company_branch_uuid.to_externel_uuid()])
            .await
            .log()?;

        let company_uuid_raw: Option<Uuid> = row.try_get(0).log()?;
        let company_uuid = match company_uuid_raw {
            Some(uuid) => UuidType(uuid.into_bytes()).into(),
            None => return Err("Branch not found".into()),
        };

        let accounts_json: Value = row.try_get(1).log()?;
        let mut accounts = Vec::new();
        if let Some(arr) = accounts_json.as_array() {
            for obj in arr {
                let obj = obj.as_object().ok_or("Invalid account object")?;
                let row_uuid_str =
                    obj.get("rowid").and_then(|v| v.as_str()).ok_or("Missing rowid")?;
                let row_uuid_parsed = Uuid::parse_str(row_uuid_str).log()?;
                let row_uuid = UuidType(row_uuid_parsed.into_bytes()).into();

                let is_debit = obj.get("is_debit").and_then(|v| v.as_bool()).unwrap_or(false);
                let is_permanent_account =
                    obj.get("is_permanent_account").and_then(|v| v.as_bool()).unwrap_or(false);
                let account_name =
                    obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let notes = obj.get("notes").and_then(|v| v.as_str()).map(String::from);
                let unit_of_measurement_of_quantity = obj
                    .get("unit_of_measurement_of_quantity")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                accounts.push(use_cases::get_all_accounts_for_branch::Account {
                    row_uuid,
                    is_debit,
                    is_permanent_account,
                    account_name,
                    notes,
                    unit_of_measurement_of_quantity,
                });
            }
        }

        let flows_json: Value = row.try_get(2).log()?;
        let mut accounts_for_branch = Vec::new();
        if let Some(arr) = flows_json.as_array() {
            for obj in arr {
                let obj = obj.as_object().ok_or("Invalid flow object")?;
                let row_uuid_str =
                    obj.get("rowid").and_then(|v| v.as_str()).ok_or("Missing rowid")?;
                let row_uuid_parsed = Uuid::parse_str(row_uuid_str).log()?;
                let row_uuid = UuidType(row_uuid_parsed.into_bytes()).into();

                let account_uuid_str =
                    obj.get("account").and_then(|v| v.as_str()).ok_or("Missing account")?;
                let account_uuid_parsed = Uuid::parse_str(account_uuid_str).log()?;
                let account_uuid = UuidType(account_uuid_parsed.into_bytes()).into();

                let outflow_type_str =
                    obj.get("outflow_type").and_then(|v| v.as_str()).unwrap_or("Manual");
                let outflow_type =
                    accounting_stuff::OutFlowType::from_str(outflow_type_str).log()?;

                let inflow_type_str =
                    obj.get("inflow_type").and_then(|v| v.as_str()).unwrap_or("Manual");
                let inflow_type = accounting_stuff::InFlowType::from_str(inflow_type_str).log()?;

                accounts_for_branch.push(
                    use_cases::get_all_accounts_for_branch::AccountForBranch {
                        row_uuid,
                        account_uuid,
                        outflow_type,
                        inflow_type,
                    },
                );
            }
        }

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
    use crate::utility::test_helper::test_query_helper;

    #[tokio::test]
    async fn test_query_string_directly() {
        test_query_helper(READ_QUERY).await.unwrap();
    }
}
