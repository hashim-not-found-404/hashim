use crate::utility::db_client;
use crate::utility::utils::MyUuidConverter1;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types::DatabaseRead;
use my_core::utility::traits::DynamicError;
use my_core::utility::utils::LogError;
use uuid::Uuid;

const READ_QUERY: &str = "SELECT rowid,pass,name FROM accounting_app.user WHERE id = $1 LIMIT 1;";

pub struct S;

impl cases::sign_in::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = db_client::S;
    type Input = cases::sign_in::ReadInput;
    type Output = cases::sign_in::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        input: &Self::Input,
    ) -> Result<Self::Output, DynamicError> {
        let stmt = db.client.prepare_cached(READ_QUERY).await.log()?;
        let row = db.client.query_opt(&stmt, &[&input.user_id]).await.log()?;

        match row {
            Some(row) => {
                let row_id = row.try_get::<_, Uuid>(0).log()?;
                let hashed_password = row.try_get::<_, String>(1).log()?;
                let name = row.try_get::<_, Option<String>>(2).log()?;

                let a = cases::sign_in::ReadOutput {
                    user_rowid_and_password_hash_and_name: Some((
                        row_id.to_uuid(),
                        hashed_password,
                        name,
                    )),
                };
                Ok(a)
            }
            None => {
                Ok(cases::sign_in::ReadOutput {
                    user_rowid_and_password_hash_and_name: None,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::test_helper::test_query_helper;

    #[tokio::test]
    async fn test_query_string_directly() {
        test_query_helper(READ_QUERY).await.unwrap();
    }
}
