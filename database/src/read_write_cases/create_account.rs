use crate::utility::db_transaction;
use crate::utility::utils::MyUuidConverter;
use my_core::domain::cases;
use my_core::domain::utility::types::DatabaseRead;
use my_core::domain::utility::types::Role;
use my_core::server::utility::server_traits;
use my_core::utility::traits::DynamicError;
use my_core::utility::utils::LogError;
use std::str::FromStr;

const READ_QUERY: &str = "
    WITH user_roles AS (
        SELECT array_agg(role) as roles
        FROM accounting_app.access_control_for_company
        WHERE data_group = $1 AND user_ = $2
    ),
    checks AS (
        SELECT
            EXISTS(SELECT 1 FROM accounting_app.company WHERE rowid = $1) AS company_exists,
            EXISTS(SELECT 1 FROM accounting_app.account WHERE rowid = $3) AS new_uuid_used,
            EXISTS(SELECT 1 FROM accounting_app.account
                  WHERE belong_to_company = $1 AND name = $4) AS account_name_used
    )
    SELECT
        COALESCE((SELECT roles FROM user_roles), '{}'::text[]) AS roles,
        (SELECT company_exists FROM checks) AS company_exists,
        (SELECT new_uuid_used FROM checks) AS new_uuid_used,
        (SELECT account_name_used FROM checks) AS account_name_used
";

pub struct S;

impl cases::create_account::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = db_transaction::S<'a>;
    type Input = cases::create_account::ReadInput;
    type Output = cases::create_account::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        input: &Self::Input,
    ) -> Result<Self::Output, DynamicError> {
        let stmt = db.txn.prepare_cached(READ_QUERY).await.log()?;
        let row = db
            .txn
            .query_one(&stmt, &[
                &input.belong_to_company.to_externel_uuid(),
                &input.user_uuid.to_externel_uuid(),
                &input.new_uuid.to_externel_uuid(),
                &input.account_name,
            ])
            .await
            .log()?;

        let role_strings: Vec<String> = row.try_get(0).log()?;
        let user_roles = role_strings
            .into_iter()
            .map(|s| Role::from_str(&s))
            .collect::<Result<Vec<_>, _>>()
            .log()?;

        Ok(cases::create_account::ReadOutput {
            is_company_uuid_exist: row.try_get(1).log()?,
            is_new_uuid_used: row.try_get(2).log()?,
            user_roles,
            is_account_name_used: row.try_get(3).log()?,
        })
    }
}

const WRITE_QUERY: &str = "
    INSERT INTO accounting_app.account (
        rowid,
        is_debit,
        is_permanent_account,
        name,
        notes,
        belong_to_company,
        unit_of_measurement_of_quantity
    ) VALUES ($1, $2, $3, $4, $5, $6, $7)
";

impl server_traits::DatabaseWrite for S {
    type Db<'a> = db_transaction::S<'a>;
    type Input = cases::create_account::Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<(), DynamicError> {
        let stmt = txn.txn.prepare_cached(WRITE_QUERY).await.log()?;
        txn.txn
            .execute(&stmt, &[
                &input.new_uuid.to_externel_uuid(),
                &input.is_debit,
                &input.is_permanent_account,
                &input.account_name,
                &input.notes,
                &input.belong_to_company.to_externel_uuid(),
                &input.unit_of_measurement_of_quantity,
            ])
            .await
            .log()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::test_helper::test_query_helper;

    #[tokio::test]
    async fn test_query_string_directly() {
        test_query_helper(READ_QUERY).await.unwrap();
        test_query_helper(WRITE_QUERY).await.unwrap();
    }
}
