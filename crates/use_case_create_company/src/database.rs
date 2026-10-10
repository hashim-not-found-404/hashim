use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use database::db_transaction;
use database::utils::MyUuidConverter;
use kernel::types::DatabaseRead;
use kernel::types::DatabaseWrite;
use utility::types::LogError;

const READ_QUERY: &str = "
    SELECT EXISTS(SELECT 1 FROM accounting_app.company WHERE rowid = $1)
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
            .query_one(&stmt, &[&input.new_uuid.to_externel_uuid()])
            .await
            .log()?;

        Ok(ReadOutput {
            is_new_uuid_used: row.try_get(0).log()?,
        })
    }
}

const WRITE_QUERY: &str = "
    WITH company_insert AS (
        INSERT INTO accounting_app.company (rowid, name, currency)
        VALUES ($1, $2, $3)
        RETURNING rowid
    )
    INSERT INTO accounting_app.access_control_for_company (rowid, data_group, user_, role)
    SELECT rowid, rowid, $4, $5 FROM company_insert
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
                    &input.company_name,
                    &input.currency.as_str(),
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
