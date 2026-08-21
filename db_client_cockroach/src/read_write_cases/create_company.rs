use crate::utility::db_transaction;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types::DatabaseRead;
use my_core::utility::traits;
use my_core::utility::utils::LogError;

const QUERY1: &str = "SELECT EXISTS(SELECT 1 FROM accounting_app.company WHERE rowid = $1)";

pub struct S;

impl cases::create_company::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = db_transaction::S<'a>;
    type Error = traits::DynamicError;
    type ReadInput = cases::create_company::ReadInput;
    type ReadOutput = cases::create_company::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &Self::ReadInput,
    ) -> Result<Self::ReadOutput, Self::Error> {
        let stmt = db.txn.prepare_cached(QUERY1).await.log()?;
        let row =
            db.txn.query_one(&stmt, &[&read_input.new_uuid.to_externel_uuid()]).await.log()?;

        let exists: bool = row.try_get(0).log()?;
        Ok(cases::create_company::ReadOutput {
            is_new_uuid_used: exists,
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
