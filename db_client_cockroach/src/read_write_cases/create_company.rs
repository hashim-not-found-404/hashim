use crate::utility::db_transaction;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::utility::traits;
use my_core::utility::utils::LogError;

pub struct S;

impl cases::create_company::DatabaseRead for S {
    type Db<'a> = db_transaction::S<'a>;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::create_company::ReadInput,
    ) -> Result<cases::create_company::ReadOutput, traits::DynamicError> {
        let query = "SELECT EXISTS(SELECT 1 FROM accounting_app.company WHERE rowid = $1)";
        let stmt = db.txn.prepare_cached(query).await.log()?;
        let row = db
            .txn
            .query_one(&stmt, &[&read_input.new_uuid.to_externel_uuid()])
            .await
            .log()?;

        let exists: bool = row.try_get(0).log()?;
        Ok(cases::create_company::ReadOutput {
            is_new_uuid_used: exists,
        })
    }
}
