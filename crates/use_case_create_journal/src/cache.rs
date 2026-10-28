use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use cache::cache_adapter;
use cache::utils::MyUuidConverter;
use kernel::types::DatabaseRead;
use kernel::types::DatabaseWrite;
use kernel::types::Role;
use rusqlite::params;
use serde_json::Value;
use serde_json::from_str;
use std::str::FromStr;

const READ_QUERY: &str = "
    SELECT
        EXISTS(SELECT 1 FROM journal WHERE rowid = ?1) AS new_uuid_used,
        EXISTS(SELECT 1 FROM company WHERE rowid = ?2) AS company_exists,
        EXISTS(SELECT 1 FROM journal WHERE company_belong = ?2 AND name = ?4) AS name_used,
        (SELECT json_group_array(role) FROM access_control_for_company
            WHERE data_group = ?2 AND user_ = ?3) AS roles_json
";

const WRITE_QUERY: &str = "
    INSERT OR REPLACE INTO journal (rowid, company_belong, name, is_it_contain_quantity)
    VALUES (?1, ?2, ?3, ?4)
";

pub struct CacheOp;

impl DatabaseRead for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db<'_>, input: &Self::Input) -> Result<Self::Output> {
        let mut stmt = db.tables_db.prepare(READ_QUERY)?;
        let row = stmt.query_row(
            params![
                input.new_uuid.to_string(),
                input.company_belong.to_string(),
                input.user_uuid.to_string(),
                &input.name,
            ],
            |row| {
                let new_uuid_used: bool = row.get(0)?;
                let company_exists: bool = row.get(1)?;
                let name_used: bool = row.get(2)?;
                let roles_json: String = row.get(3)?;
                Ok((new_uuid_used, company_exists, name_used, roles_json))
            },
        )?;

        let roles_value: Value = from_str(&row.3).unwrap_or(Value::Array(vec![]));
        let user_roles: Vec<Role> = roles_value
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| Role::from_str(s).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(ReadOutput {
            user_roles,
            is_new_uuid_used: row.0,
            is_company_exist: row.1,
            is_name_used: row.2,
        })
    }
}

impl DatabaseWrite for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<()> {
        txn.tables_db.execute(
            WRITE_QUERY,
            params![
                input.new_uuid.to_string(),
                input.company_belong.to_string(),
                &input.name,
                input.is_it_contain_quantity,
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
