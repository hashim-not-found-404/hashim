use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use cache::cache_adapter;
use cache::utils::MyUuidConverter;
use kernel::types::DatabaseRead;
use kernel::types::DatabaseWrite;
use rusqlite::params;

const READ_QUERY: &str = "SELECT
    EXISTS(SELECT 1 FROM user WHERE rowid = ?1),
    EXISTS(SELECT 1 FROM user WHERE id = ?2)";

pub struct CacheOp;

impl DatabaseRead for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db<'_>, input: &Self::Input) -> Result<Self::Output> {
        let query = READ_QUERY;

        let a = db.tables_db.query_one(
            query,
            params![input.new_uuid.to_string(), input.user_id],
            |row| {
                Ok(ReadOutput {
                    is_new_uuid_exist: row.get(0)?,
                    is_user_id_exist: row.get(1)?,
                })
            },
        )?;

        Ok(a)
    }
}

const WRITE_QUERY: &str = "INSERT INTO user (rowid, id, name, jwt) VALUES (?1, ?2, ?3, ?4)";

impl DatabaseWrite for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<()> {
        txn.tables_db.execute(
            WRITE_QUERY,
            params![
                input.new_uuid.to_string(),
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
