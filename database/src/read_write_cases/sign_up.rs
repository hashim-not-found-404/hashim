use crate::utility::db_transaction;
use crate::utility::utils::MyUuidConverter;
use my_core::domain::use_cases;
use my_core::domain::utility::types::DatabaseRead;
use my_core::server::utility::server_traits;
use my_core::utility::traits::DynamicError;
use my_core::utility::utils::LogError;

const READ_QUERY: &str = "
     SELECT
         EXISTS(SELECT 1 FROM accounting_app.user WHERE rowid = $1) AS uuid_exists,
         EXISTS(SELECT 1 FROM accounting_app.user WHERE id = $2) AS user_id_exists
";

pub struct S;

impl use_cases::sign_up::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = db_transaction::S<'a>;
    type Input = use_cases::sign_up::ReadInput;
    type Output = use_cases::sign_up::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        input: &Self::Input,
    ) -> Result<Self::Output, DynamicError> {
        let stmt = db.txn.prepare_cached(READ_QUERY).await.log()?;
        let row = db
            .txn
            .query_one(&stmt, &[&input.new_uuid.to_externel_uuid(), &input.user_id])
            .await
            .log()?;

        let a = use_cases::sign_up::ReadOutput {
            is_new_uuid_exist: row.try_get("uuid_exists").log()?,
            is_user_id_exist:  row.try_get("user_id_exists").log()?,
        };
        Ok(a)
    }
}

const WRITE_QUERY: &str =
    "INSERT INTO accounting_app.user (rowid, id, pass, name) VALUES ($1, $2, $3, $4)";

impl server_traits::DatabaseWrite for S {
    type Db<'a> = db_transaction::S<'a>;
    type Input = use_cases::sign_up::Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<(), DynamicError> {
        let stmt = txn.txn.prepare_cached(WRITE_QUERY).await.log()?;

        txn.txn
            .execute(&stmt, &[
                &input.new_uuid.to_externel_uuid(),
                &input.user_id,
                &input.hashed_password,
                &input.user_name,
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
