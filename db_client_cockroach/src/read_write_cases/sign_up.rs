use crate::utility::db_transaction;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::utility::traits;
use my_core::utility::utils::LogError;

pub struct S;

impl cases::sign_up::DatabaseRead for S {
    type Db<'a> = db_transaction::S<'a>;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::sign_up::ReadInput,
    ) -> Result<cases::sign_up::ReadOutput, traits::DynamicError> {
        let query = "
             SELECT
                 EXISTS(SELECT 1 FROM accounting_app.user WHERE rowid = $1) AS uuid_exists,
                 EXISTS(SELECT 1 FROM accounting_app.user WHERE id = $2) AS user_id_exists
         ";

        let stmt = db.txn.prepare_cached(query).await.log()?;
        let row = db
            .txn
            .query_one(
                &stmt,
                &[&read_input.new_uuid.to_externel_uuid(), &read_input.user_id],
            )
            .await
            .log()?;

        let a = cases::sign_up::ReadOutput {
            is_new_uuid_exist: row.try_get("uuid_exists").log()?,
            is_user_id_exist: row.try_get("user_id_exists").log()?,
        };
        Ok(a)
    }
}
