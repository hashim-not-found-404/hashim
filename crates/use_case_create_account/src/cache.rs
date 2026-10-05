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
use std::str::FromStr;

const READ_QUERY1: &str =
    "SELECT role FROM access_control_for_company WHERE data_group = ?1 AND user_ = ?2";
const READ_QUERY2: &str = "SELECT 1 FROM company WHERE rowid = ?1";
const READ_QUERY3: &str = "SELECT 1 FROM account WHERE rowid = ?1";
const READ_QUERY4: &str = "SELECT 1 FROM account WHERE belong_to_company = ?1 AND name = ?2";

const WRITE_QUERY: &str = "
    INSERT OR REPLACE INTO account (
        rowid,
        is_debit,
        is_permanent_account,
        name,
        unit_of_measurement_of_quantity,
        belong_to_company
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
";

pub struct CacheOp;

impl DatabaseRead for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db<'_>, input: &Self::Input) -> Result<Self::Output> {
        let mut stmt = db.tables_db.prepare(READ_QUERY1)?;
        let roles_iter = stmt.query_map(
            params![
                input.belong_to_company.to_string(),
                input.user_uuid.to_string()
            ],
            |row| {
                let role_str: String = row.get(0)?;
                let role = Role::from_str(role_str.as_str()).unwrap();
                Ok(role)
            },
        )?;
        let user_roles: Vec<Role> = roles_iter.map(|r| r.unwrap()).collect();
        let mut stmt = db.tables_db.prepare(READ_QUERY2)?;
        let is_company_uuid_exist = stmt.exists(params![input.belong_to_company.to_string()])?;
        let mut stmt = db.tables_db.prepare(READ_QUERY3)?;
        let is_new_uuid_used = stmt.exists(params![input.new_uuid.to_string()])?;
        let mut stmt = db.tables_db.prepare(READ_QUERY4)?;
        let is_account_name_used = stmt.exists(params![
            input.belong_to_company.to_string(),
            &input.account_name
        ])?;

        Ok(ReadOutput {
            is_company_uuid_exist,
            is_new_uuid_used,
            user_roles,
            is_account_name_used,
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
                input.is_debit,
                input.is_permanent_account,
                &input.account_name,
                &input.unit_of_measurement_of_quantity,
                input.belong_to_company.to_string(),
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
        test_query_helper_for_tables_schema(READ_QUERY1).unwrap();
        test_query_helper_for_tables_schema(READ_QUERY2).unwrap();
        test_query_helper_for_tables_schema(READ_QUERY3).unwrap();
        test_query_helper_for_tables_schema(READ_QUERY4).unwrap();
        test_query_helper_for_tables_schema(WRITE_QUERY).unwrap();
    }
}
