use crate::utility::db_transaction;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types::DatabaseRead;
use my_core::server::utility::server_traits;
use my_core::utility::traits;
use my_core::utility::utils::LogError;

const QUERY1: &str = "SELECT EXISTS(SELECT 1 FROM accounting_app.company WHERE rowid = $1)";

pub struct S;

impl cases::create_company::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = db_transaction::S<'a>;
    type Error = traits::DynamicError;
    type Input = cases::create_company::ReadInput;
    type Output = cases::create_company::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        let stmt = db.txn.prepare_cached(QUERY1).await.log()?;
        let row =
            db.txn.query_one(&stmt, &[&read_input.new_uuid.to_externel_uuid()]).await.log()?;

        let exists: bool = row.try_get(0).log()?;
        Ok(cases::create_company::ReadOutput {
            is_new_uuid_used: exists,
        })
    }
}

impl server_traits::DatabaseWrite for S {
    type Input = cases::create_company::Ok;
    type Txn<'a> = db_transaction::S<'a>;

    async fn write(
        txn: &mut Self::Txn<'_>,
        input: &Self::Input,
    ) -> Result<(), traits::DynamicError> {
        let query = "
                WITH company_insert AS (
                    INSERT INTO accounting_app.company (rowid, name, currency)
                    VALUES ($1, $2, $3)
                    RETURNING 1
                )
                INSERT INTO accounting_app.access_control_for_company (rowid, data_group, user_, role)
                VALUES ($1, $1, $4, $5)
                ;";

        let stmt = txn.txn.prepare_cached(query).await.log()?;
        txn.txn
            .execute(&stmt, &[
                &input.new_uuid.to_externel_uuid(),
                &input.company_name,
                &input.currency.as_str(),
                &input.user_uuid.to_externel_uuid(),
                &input.role.as_str(),
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
        test_query_helper(QUERY1).await.unwrap();
    }
}
