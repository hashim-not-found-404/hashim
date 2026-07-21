use crate::read_write_cases::utils::{db_transaction, utils::MyUuidConverter};
use my_core::{
    accounting_domain::cases::{self, utility::types},
    utility::{traits, utils::LogError},
};
use std::str::FromStr;

pub struct S;

impl cases::create_account::DatabaseRead for S {
    type Db<'a> = db_transaction::S<'a>;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::create_account::ReadInput,
    ) -> Result<cases::create_account::ReadOutput, traits::DynamicError> {
        let query = "
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

        let stmt = db.txn.prepare_cached(query).await.log()?;
        let row = db
            .txn
            .query_one(
                &stmt,
                &[
                    &read_input.belong_to_company.to_externel_uuid(),
                    &read_input.user_uuid.to_externel_uuid(),
                    &read_input.new_uuid.to_externel_uuid(),
                    &read_input.account_name,
                ],
            )
            .await
            .log()?;

        let role_strings: Vec<String> = row.try_get(0).log()?;
        let user_roles = role_strings
            .into_iter()
            .map(|s| types::Role::from_str(&s))
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
