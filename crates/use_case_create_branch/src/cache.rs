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

const QUERY1: &str =
    "SELECT role FROM access_control_for_company WHERE data_group = ?1 AND user_ = ?2";
const QUERY2: &str = "SELECT 1 FROM company WHERE rowid = ?1";
const QUERY3: &str = "SELECT 1 FROM company_branch WHERE rowid = ?1";
const QUERY4: &str = "SELECT 1 FROM company_branch WHERE company_belong = ?1 AND name = ?2";

pub struct CacheOp;

impl DatabaseRead for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db<'_>, input: &Self::Input) -> Result<Self::Output> {
        let mut stmt = db.tables_db.prepare(QUERY1)?;
        let roles_iter = stmt.query_map(
            params![
                input.company_belong.to_string(),
                input.user_uuid.to_string()
            ],
            |row| {
                let role_str: String = row.get(0)?;
                let role = Role::from_str(role_str.as_str()).unwrap();
                Ok(role)
            },
        )?;
        let user_roles: Vec<Role> = roles_iter.map(|r| r.unwrap()).collect();

        let mut stmt = db.tables_db.prepare(QUERY2)?;
        let is_company_exist = stmt.exists(params![input.company_belong.to_string()])?;

        let mut stmt = db.tables_db.prepare(QUERY3)?;
        let is_new_uuid_used = stmt.exists(params![input.new_uuid.to_string()])?;

        let mut stmt = db.tables_db.prepare(QUERY4)?;
        let is_branch_name_used = stmt.exists(params![
            input.company_belong.to_string(),
            &input.branch_name
        ])?;

        Ok(ReadOutput {
            user_roles,
            is_new_uuid_used,
            is_company_exist,
            is_branch_name_used,
        })
    }
}

const WRITE_BRANCH_QUERY: &str = "
    INSERT OR REPLACE INTO company_branch (
        rowid, company_belong, name, currency,
        location_latitude, location_longitude
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
";

const WRITE_ACCESS_QUERY: &str = "
    INSERT OR REPLACE INTO access_control_for_company_branch (rowid, data_group, user_, role)
    VALUES (?1, ?2, ?3, ?4)
";

impl DatabaseWrite for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<()> {
        let branch_uuid = input.new_uuid.to_string();
        let company_uuid = input.company_belong.to_string();

        txn.tables_db.execute(
            WRITE_BRANCH_QUERY,
            params![
                &branch_uuid,
                &company_uuid,
                &input.branch_name,
                input.currency.as_str(),
                input.location.latitude,
                input.location.longitude,
            ],
        )?;

        txn.tables_db.execute(
            WRITE_ACCESS_QUERY,
            params![
                &branch_uuid,
                &branch_uuid,
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
        test_query_helper_for_tables_schema(QUERY1).unwrap();
        test_query_helper_for_tables_schema(QUERY2).unwrap();
        test_query_helper_for_tables_schema(QUERY3).unwrap();
        test_query_helper_for_tables_schema(QUERY4).unwrap();
        test_query_helper_for_tables_schema(WRITE_BRANCH_QUERY).unwrap();
        test_query_helper_for_tables_schema(WRITE_ACCESS_QUERY).unwrap();
    }
}
