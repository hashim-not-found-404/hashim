use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Context;
use anyhow::Result;
use database::db_transaction;
use database::utils::MyUuidConverter;
use kernel::types::DatabaseRead;
use kernel::types::DatabaseWrite;
use kernel::types::Role;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::str::FromStr;
use utility::types::LogError;

const READ_QUERY: &str = "
    WITH user_roles AS (
        SELECT array_agg(role) AS roles
        FROM accounting_app.access_control_for_company
        WHERE data_group = $2 AND user_ = $3
    ),
    checks AS (
        SELECT
            EXISTS(SELECT 1 FROM accounting_app.company_branch WHERE rowid = $1) AS new_uuid_used,
            EXISTS(SELECT 1 FROM accounting_app.company WHERE rowid = $2) AS company_exists,
            EXISTS(SELECT 1 FROM accounting_app.company_branch
                  WHERE company_belong = $2 AND name = $4) AS branch_name_used
    )
    SELECT
        COALESCE((SELECT roles FROM user_roles), '{}'::text[]) AS roles,
        (SELECT new_uuid_used FROM checks) AS new_uuid_used,
        (SELECT company_exists FROM checks) AS company_exists,
        (SELECT branch_name_used FROM checks) AS branch_name_used
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
                    &input.branch_name,
                ],
            )
            .await
            .log()?;

        let role_strings: Vec<String> = row.try_get(0).log()?;
        let user_roles = role_strings
            .into_iter()
            .map(|s| Role::from_str(&s))
            .collect::<Result<Vec<_>, _>>()
            .log()?;

        Ok(ReadOutput {
            user_roles,
            is_new_uuid_used: row.try_get(1).log()?,
            is_company_exist: row.try_get(2).log()?,
            is_branch_name_used: row.try_get(3).log()?,
        })
    }
}

const WRITE_QUERY: &str = "
    WITH branch_insert AS (
        INSERT INTO accounting_app.company_branch (
            rowid, company_belong, name, currency,
            location_latitude, location_longitude
        ) VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING rowid
    )
    INSERT INTO accounting_app.access_control_for_company_branch (rowid, data_group, user_, role)
    SELECT rowid, rowid, $7, $8 FROM branch_insert
";

impl DatabaseWrite for DataBaseOp {
    type Db<'a> = db_transaction::S<'a>;
    type Input = Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<()> {
        let stmt = txn.txn.prepare_cached(WRITE_QUERY).await.log()?;

        let latitude = Decimal::from_f64(input.location.latitude)
            .context("latitude is not a finite decimal")?;
        let longitude = Decimal::from_f64(input.location.longitude)
            .context("longitude is not a finite decimal")?;

        txn.txn
            .execute(
                &stmt,
                &[
                    &input.new_uuid.to_externel_uuid(),
                    &input.company_belong.to_externel_uuid(),
                    &input.branch_name,
                    &input.currency.as_str(),
                    &latitude,
                    &longitude,
                    &input.user_uuid.to_externel_uuid(),
                    &input.role.as_str(),
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
