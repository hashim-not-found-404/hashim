use crate::utility::db_transaction;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types::DatabaseRead;
use my_core::utility::traits;
use my_core::utility::utils::LogError;

const QUERY1: &str = "
     SELECT
         EXISTS(SELECT 1 FROM accounting_app.user WHERE rowid = $1) AS uuid_exists,
         EXISTS(SELECT 1 FROM accounting_app.user WHERE id = $2) AS user_id_exists
";

pub struct S;

impl cases::sign_up::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = db_transaction::S<'a>;
    type Error = traits::DynamicError;
    type ReadInput = cases::sign_up::ReadInput;
    type ReadOutput = cases::sign_up::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &Self::ReadInput,
    ) -> Result<Self::ReadOutput, Self::Error> {
        let stmt = db.txn.prepare_cached(QUERY1).await.log()?;
        let row = db
            .txn
            .query_one(&stmt, &[&read_input.new_uuid.to_externel_uuid(), &read_input.user_id])
            .await
            .log()?;

        let a = cases::sign_up::ReadOutput {
            is_new_uuid_exist: row.try_get("uuid_exists").log()?,
            is_user_id_exist:  row.try_get("user_id_exists").log()?,
        };
        Ok(a)
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
