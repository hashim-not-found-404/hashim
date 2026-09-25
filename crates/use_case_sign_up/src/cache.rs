use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use cache::cache_adapter;
use cache::utils::MyUuidConverter;
use kernel::types::DatabaseRead;
use kernel::types::DatabaseWrite;
use rusqlite::params;

const QUERY: &str = "SELECT
    EXISTS(SELECT 1 FROM user WHERE rowid = ?1),
    EXISTS(SELECT 1 FROM user WHERE id = ?2)";

pub struct S;

impl DatabaseRead for S {
    type Db<'a> = cache_adapter::S;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db<'_>, input: &Self::Input) -> Result<Self::Output> {
        let query = QUERY;

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

impl DatabaseWrite for S {
    type Db<'a> = cache_adapter::S;
    type Input = Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cache::test_helper::test_query_helper_for_tables_schema;

    #[test]
    fn test_query_string_directly() {
        test_query_helper_for_tables_schema(QUERY).unwrap();
    }
}
