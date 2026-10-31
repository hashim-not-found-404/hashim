use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use std::str::FromStr;
use use_case_prelude_database::*;

const READ_QUERY: &str = "
    WITH
    new_uuid_check AS (
        SELECT (
            EXISTS(SELECT 1 FROM accounting_app.journal_with_quantity WHERE rowid = $1)
            OR EXISTS(SELECT 1 FROM accounting_app.journal_without_quantity WHERE rowid = $1)
        ) AS used
    ),
    company_check AS (
        SELECT EXISTS(SELECT 1 FROM accounting_app.company WHERE rowid = $2) AS exists
    ),
    name_check AS (
        SELECT (
            EXISTS(SELECT 1 FROM accounting_app.journal_with_quantity
                   WHERE company_belong = $2 AND name = $4)
            OR EXISTS(SELECT 1 FROM accounting_app.journal_without_quantity
                   WHERE company_belong = $2 AND name = $4)
        ) AS used
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

const WRITE_JOURNAL_WITH_QUANTITY_QUERY: &str = "
    INSERT INTO accounting_app.journal_with_quantity (rowid, company_belong, name)
    VALUES ($1, $2, $3)
";

const WRITE_JOURNAL_WITHOUT_QUANTITY_QUERY: &str = "
    INSERT INTO accounting_app.journal_without_quantity (rowid, company_belong, name)
    VALUES ($1, $2, $3)
";

pub struct DataBaseOp;

impl DatabaseRead for DataBaseOp {
    type Db<'a> = DbTransactionAdapter<'a>;
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

impl DatabaseWrite for DataBaseOp {
    type Db<'a> = DbTransactionAdapter<'a>;
    type Input = Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<()> {
        let query = if input.is_it_contain_quantity {
            WRITE_JOURNAL_WITH_QUANTITY_QUERY
        } else {
            WRITE_JOURNAL_WITHOUT_QUANTITY_QUERY
        };

        let stmt = txn.txn.prepare_cached(query).await.log()?;
        txn.txn
            .execute(
                &stmt,
                &[
                    &input.new_uuid.to_externel_uuid(),
                    &input.company_belong.to_externel_uuid(),
                    &input.name,
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

    #[tokio::test]
    async fn test_query_string_directly() {
        test_query_helper(READ_QUERY).await.unwrap();
        test_query_helper(WRITE_JOURNAL_WITH_QUANTITY_QUERY)
            .await
            .unwrap();
        test_query_helper(WRITE_JOURNAL_WITHOUT_QUANTITY_QUERY)
            .await
            .unwrap();
    }
}
