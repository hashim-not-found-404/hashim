use crate::utility::db_transaction;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types;
use my_core::utility::traits;
use my_core::utility::utils::LogError;
use std::str::FromStr;

const QUERY1: &str = "
    WITH
    -- 1. Get user roles for the company that owns the branch
    user_roles AS (
        SELECT array_agg(acf.role) as roles
        FROM accounting_app.access_control_for_company acf
        JOIN accounting_app.company_branch cb ON acf.data_group = cb.company_belong
        WHERE cb.rowid = $1 AND acf.user_ = $2
    ),
    -- 2. Check existence and duplicate conditions
    checks AS (
        SELECT
            EXISTS(SELECT 1 FROM accounting_app.account_flow_type WHERE rowid = $3) AS new_uuid_used,
            EXISTS(SELECT 1 FROM accounting_app.account WHERE rowid = $4) AS account_exists,
            EXISTS(SELECT 1 FROM accounting_app.company_branch WHERE rowid = $1) AS branch_exists,
            EXISTS(SELECT 1 FROM accounting_app.account_flow_type
                  WHERE account = $4 AND company_branch = $1) AS account_branch_used
    )
    SELECT
        COALESCE((SELECT roles FROM user_roles), '{}'::text[]) as roles,
        (SELECT new_uuid_used FROM checks) as new_uuid_used,
        (SELECT account_exists FROM checks) as account_exists,
        (SELECT branch_exists FROM checks) as branch_exists,
        (SELECT account_branch_used FROM checks) as account_branch_used
";

pub struct S;

impl cases::create_account_for_branch::DatabaseRead for S {
    type Db<'a> = db_transaction::S<'a>;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::create_account_for_branch::ReadInput,
    ) -> Result<cases::create_account_for_branch::ReadOutput, traits::DynamicError> {
        let row = db
            .txn
            .query_one(QUERY1, &[
                &read_input.belong_to_company_branch.to_externel_uuid(),
                &read_input.user_uuid.to_externel_uuid(),
                &read_input.new_uuid.to_externel_uuid(),
                &read_input.belong_to_account.to_externel_uuid(),
            ])
            .await
            .log()?;

        let role_strings: Vec<String> = row.try_get(0).log()?;
        let user_roles = role_strings
            .into_iter()
            .map(|s| types::Role::from_str(&s))
            .collect::<Result<Vec<_>, _>>()
            .log()?;

        Ok(cases::create_account_for_branch::ReadOutput {
            user_roles,
            is_new_uuid_used: row.try_get(1).log()?,
            is_account_uuid_exist: row.try_get(2).log()?,
            is_company_branch_exist: row.try_get(3).log()?,
            is_account_uuid_with_company_branch_used: row.try_get(4).log()?,
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
    }
}
