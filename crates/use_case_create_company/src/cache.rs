use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use cache::cache_adapter;
use cache::utils::MyUuidConverter;
use kernel::types::DatabaseRead;
use kernel::types::DatabaseWrite;
use rusqlite::params;

const READ_QUERY: &str = "SELECT 1 FROM company WHERE rowid = ?1";

pub struct CacheOp;

impl DatabaseRead for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db<'_>, input: &Self::Input) -> Result<Self::Output> {
        let mut stmt = db.tables_db.prepare(READ_QUERY)?;
        let is_new_uuid_used = stmt.exists(params![input.new_uuid.to_string()])?;

        Ok(ReadOutput { is_new_uuid_used })
    }
}

const WRITE_COMPANY_QUERY: &str =
    "INSERT OR REPLACE INTO company (rowid, name, currency) VALUES (?1, ?2, ?3)";

const WRITE_ACCESS_QUERY: &str = "
    INSERT OR REPLACE INTO access_control_for_company (rowid, data_group, user_, role)
    VALUES (?1, ?2, ?3, ?4)
";

impl DatabaseWrite for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<()> {
        let company_uuid = input.new_uuid.to_string();

        txn.tables_db.execute(
            WRITE_COMPANY_QUERY,
            params![&company_uuid, &input.company_name, input.currency.as_str(),],
        )?;

        txn.tables_db.execute(
            WRITE_ACCESS_QUERY,
            params![
                &company_uuid,
                &company_uuid,
                input.user_uuid.to_string(),
                input.role.as_str(),
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
        test_query_helper_for_tables_schema(WRITE_COMPANY_QUERY).unwrap();
        test_query_helper_for_tables_schema(WRITE_ACCESS_QUERY).unwrap();
    }
}
