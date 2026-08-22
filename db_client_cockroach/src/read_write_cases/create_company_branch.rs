use crate::utility::db_transaction;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types;
use my_core::accounting_domain::utility::types::DatabaseRead;
use my_core::server::utility::server_traits;
use my_core::utility::traits::DynamicError;
use my_core::utility::utils::LogError;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::str::FromStr;

const READ_QUERY: &str = "
    WITH user_roles AS (
        SELECT array_agg(role) as roles
        FROM accounting_app.access_control_for_company
        WHERE data_group = $1 AND user_ = $2
    ),
    checks AS (
        SELECT
            EXISTS(SELECT 1 FROM accounting_app.company_branch WHERE rowid = $3) as new_uuid_exists,
            EXISTS(SELECT 1 FROM accounting_app.company WHERE rowid = $1) as company_exists,
            EXISTS(SELECT 1 FROM accounting_app.company_branch
                  WHERE company_belong = $1 AND name = $4) as branch_name_used
    )
    SELECT
        COALESCE((SELECT roles FROM user_roles), '{}'::text[]) as roles,
        (SELECT new_uuid_exists FROM checks) as new_uuid_exists,
        (SELECT company_exists FROM checks) as company_exists,
        (SELECT branch_name_used FROM checks) as branch_name_used
";

pub struct S;

impl cases::create_company_branch::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = db_transaction::S<'a>;
    type Input = cases::create_company_branch::ReadInput;
    type Output = cases::create_company_branch::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        input: &Self::Input,
    ) -> Result<Self::Output, DynamicError> {
        let row = db
            .txn
            .query_one(READ_QUERY, &[
                &input.company_belong.to_externel_uuid(),
                &input.user_uuid.to_externel_uuid(),
                &input.new_uuid.to_externel_uuid(),
                &input.branch_name,
            ])
            .await
            .log()?;

        let role_strings: Vec<String> = row.try_get(0).log()?;
        let roles = role_strings
            .into_iter()
            .map(|s| types::Role::from_str(&s))
            .collect::<Result<Vec<_>, _>>()
            .log()?;

        let a = cases::create_company_branch::ReadOutput {
            user_roles:          roles,
            is_new_uuid_used:    row.try_get(1).log()?,
            is_company_exist:    row.try_get(2).log()?,
            is_branch_name_used: row.try_get(3).log()?,
        };
        Ok(a)
    }
}

const WRITE_QUERY: &str = "
    WITH inserted_branch AS (
        INSERT INTO accounting_app.company_branch (
            rowid, company_belong, name,
            location_latitude, location_longitude, currency
        ) VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING rowid
    )
    INSERT INTO accounting_app.access_control_for_company_branch (
        rowid, data_group, user_, role
    )
    SELECT rowid, rowid, $7, $8 FROM inserted_branch
";

impl server_traits::DatabaseWrite for S {
    type Db<'a> = db_transaction::S<'a>;
    type Input = cases::create_company_branch::Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<(), DynamicError> {
        let lat = Decimal::from_f64(input.location.latitude)
            .ok_or(types::HashimError::InternalServerError)
            .log()?;
        let lng = Decimal::from_f64(input.location.longitude)
            .ok_or(types::HashimError::InternalServerError)
            .log()?;

        txn.txn
            .execute(WRITE_QUERY, &[
                &input.new_uuid.to_externel_uuid(),
                &input.company_belong.to_externel_uuid(),
                &input.branch_name,
                &lat,
                &lng,
                &input.currency.as_str(),
                &input.user_uuid.to_externel_uuid(),
                &input.role.as_str(),
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
