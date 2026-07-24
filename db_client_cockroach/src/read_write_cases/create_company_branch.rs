use crate::utility::db_transaction;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types;
use my_core::utility::traits;
use my_core::utility::utils::LogError;
use std::str::FromStr;

pub struct S;

impl cases::create_company_branch::DatabaseRead for S {
    type Db<'a> = db_transaction::S<'a>;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::create_company_branch::ReadInput,
    ) -> Result<cases::create_company_branch::ReadOutput, traits::DynamicError> {
        let query = "
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

        let row = db
            .txn
            .query_one(query, &[
                &read_input.company_belong.to_externel_uuid(),
                &read_input.user_uuid.to_externel_uuid(),
                &read_input.new_uuid.to_externel_uuid(),
                &read_input.branch_name,
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
