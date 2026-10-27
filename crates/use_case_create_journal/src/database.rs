use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use database::db_transaction;
use database::utils::MyUuidConverter;
use kernel::types::DatabaseRead;
use kernel::types::DatabaseWrite;
use kernel::types::Role;
use serde_json::Value;
use std::str::FromStr;
use utility::types::LogError;

const READ_QUERY: &str = "
    WITH
    new_uuid_check AS (
        SELECT EXISTS(SELECT 1 FROM accounting_app.journal WHERE rowid = $1) AS used
    ),
    company_check AS (
        SELECT EXISTS(SELECT 1 FROM accounting_app.company WHERE rowid = $2) AS exists
    ),
    name_check AS (
        SELECT EXISTS(SELECT 1 FROM accounting_app.journal
                      WHERE company_belong = $2 AND name = $4) AS used
    ),
    user_roles AS (
        SELECT COALESCE(jsonb_agg(role), '[]'::jsonb) AS roles
        FROM accounting_app.access_control_for_company
        WHERE data_group = $2 AND user_ = $3
    )
    SELECT
        (SELECT used   FROM new_uuid_check) AS new_uuid_used,
        (SELECT exists FROM company_check)  AS company_exists,
        (SELECT used   FROM name_check)     AS name_used,
        (SELECT roles  FROM user_roles)     AS user_roles
";

pub struct DataBaseOp;

impl DatabaseRead for DataBaseOp {
    type Db<'a> = db_transaction::S<'a>;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db<'_>, input: &Self::Input) -> Result<Self::Output> {
        let stmt = db.txn.prepare_cached(READ_QUERY).await.log()?;
        let row = db
            .txn
            .query_one(
                &stmt,
                &[
                    &input.new_uuid.to_externel_uuid(),
                    &input.company_belong.to_externel_uuid(),
                    &input.user_uuid.to_externel_uuid(),
                    &input.name,
                ],
            )
            .await
            .log()?;

        let roles_value: Value = row.try_get(3).log()?;
        let user_roles = roles_value
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| Role::from_str(s).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(ReadOutput {
            user_roles,
            is_new_uuid_used: row.try_get(0).log()?,
            is_company_exist: row.try_get(1).log()?,
            is_name_used: row.try_get(2).log()?,
        })
    }
}

const WRITE_QUERY: &str = "
    INSERT INTO accounting_app.journal (rowid, company_belong, name, is_it_contain_quantity)
    VALUES ($1, $2, $3, $4)
";

impl DatabaseWrite for DataBaseOp {
    type Db<'a> = db_transaction::S<'a>;
    type Input = Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<()> {
        let stmt = txn.txn.prepare_cached(WRITE_QUERY).await.log()?;
        txn.txn
            .execute(
                &stmt,
                &[
                    &input.new_uuid.to_externel_uuid(),
                    &input.company_belong.to_externel_uuid(),
                    &input.name,
                    &input.is_it_contain_quantity,
                ],
            )
            .await
            .log()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use database::test_helper::test_query_helper;

    #[tokio::test]
    async fn test_query_string_directly() {
        test_query_helper(READ_QUERY).await.unwrap();
        test_query_helper(WRITE_QUERY).await.unwrap();
    }
}
