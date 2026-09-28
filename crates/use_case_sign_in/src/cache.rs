use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use cache::cache_adapter;
use cache::utils::MyUuidConverter;
use cache::utils::MyUuidConverter1;
use kernel::new_types::UserUuid;
use kernel::types::DatabaseRead;
use kernel::types::DatabaseWrite;
use rusqlite::params;

const READ_QUERY: &str = "SELECT rowid, jwt, name FROM user WHERE id = ?1";

pub struct CacheOp;

impl DatabaseRead for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db<'_>, input: &Self::Input) -> Result<Self::Output> {
        let mut stmt = db.tables_db.prepare(READ_QUERY)?;

        let row = stmt
            .query_row([&input.user_id], |row| {
                let uuid_str: String = row.get(0)?;
                let jwt: Option<String> = row.get(1)?;
                let name: Option<String> = row.get(2)?;
                Ok((uuid_str, jwt.unwrap_or_default(), name))
            })
            .ok();

        let user = match row {
            Some((uuid_str, jwt, name)) => {
                let uuid = UserUuid::from(uuid_str.to_uuid()?);
                Some((uuid, jwt, name))
            }
            None => None,
        };

        Ok(ReadOutput {
            user_rowid_and_password_or_jwt_and_name: user,
        })
    }
}

const WRITE_QUERY: &str = "
    INSERT OR REPLACE INTO user (rowid, id, name, jwt)
    VALUES (?1, ?2, ?3, ?4)
";

impl DatabaseWrite for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<()> {
        txn.tables_db.execute(
            WRITE_QUERY,
            params![
                input.user_uuid.to_string(),
                input.user_id,
                input.user_name,
                input.jwt.0,
            ],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cache::test_helper::test_query_helper_for_tables_schema;

    #[test]
    fn test_query_string_directly() {
        test_query_helper_for_tables_schema(READ_QUERY).unwrap();
        test_query_helper_for_tables_schema(WRITE_QUERY).unwrap();
    }
}
