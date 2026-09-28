use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use database::db_client;
use kernel::new_types::UuidType;
use kernel::types::DatabaseRead;
use utility::types::LogError;
use uuid::Uuid;

const READ_QUERY: &str = "
    SELECT rowid, pass, name
    FROM accounting_app.user
    WHERE id = $1
    LIMIT 1
";

pub struct DataBaseOp;

impl DatabaseRead for DataBaseOp {
    type Db<'a> = db_client::S;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db<'_>, input: &Self::Input) -> Result<Self::Output> {
        let stmt = db.client.prepare_cached(READ_QUERY).await.log()?;
        let row = db.client.query_opt(&stmt, &[&input.user_id]).await.log()?;

        let user = match row {
            Some(row) => {
                let user_uuid: Uuid = row.try_get(0).log()?;
                let password_hash: String = row.try_get(1).log()?;
                let user_name: Option<String> = row.try_get(2).log()?;

                Some((
                    UuidType::from(user_uuid.into_bytes()).into(),
                    password_hash,
                    user_name,
                ))
            }
            None => None,
        };

        Ok(ReadOutput {
            user_rowid_and_password_or_jwt_and_name: user,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use database::test_helper::test_query_helper;

    #[tokio::test]
    async fn test_query_string_directly() {
        test_query_helper(READ_QUERY).await.unwrap();
    }
}
