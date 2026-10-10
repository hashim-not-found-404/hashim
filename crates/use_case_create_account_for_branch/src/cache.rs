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

const QUERY1: &str = "SELECT role FROM access_control_for_company
     WHERE data_group = (SELECT company_belong FROM company_branch WHERE rowid = ?1)
     AND user_ = ?2";
const QUERY2: &str = "SELECT 1 FROM account_flow_type WHERE rowid = ?1";
const QUERY3: &str = "SELECT 1 FROM account WHERE rowid = ?1";
const QUERY4: &str = "SELECT 1 FROM company_branch WHERE rowid = ?1";
const QUERY5: &str = "SELECT 1 FROM account_flow_type WHERE account = ?1 AND company_branch = ?2";

const WRITE_QUERY: &str = "
    INSERT OR REPLACE INTO account_flow_type (
        rowid,
        account,
        company_branch,
        outflow_type,
        inflow_type
    ) VALUES (?1, ?2, ?3, ?4, ?5)
";

pub struct CacheOp;

impl DatabaseRead for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db<'_>, input: &Self::Input) -> Result<Self::Output> {
        let branch_uuid = input.belong_to_company_branch.to_string();
        let user_uuid = input.user_uuid.to_string();
        let new_uuid = input.new_uuid.to_string();
        let account_uuid = input.belong_to_account.to_string();

        let mut stmt = db.tables_db.prepare(QUERY1)?;
        let roles_iter = stmt.query_map(params![branch_uuid, user_uuid], |row| {
            let role_str: String = row.get(0)?;
            let role = Role::from_str(&role_str).unwrap();
            Ok(role)
        })?;
        let mut user_roles = Vec::new();
        for role in roles_iter {
            user_roles.push(role?);
        }

        let mut stmt = db.tables_db.prepare(QUERY2)?;
        let is_new_uuid_used = stmt.exists(params![new_uuid])?;

        let mut stmt = db.tables_db.prepare(QUERY3)?;
        let is_account_uuid_exist = stmt.exists(params![account_uuid])?;

        let mut stmt = db.tables_db.prepare(QUERY4)?;
        let is_company_branch_exist = stmt.exists(params![branch_uuid])?;

        let mut stmt = db.tables_db.prepare(QUERY5)?;
        let is_account_uuid_with_company_branch_used =
            stmt.exists(params![account_uuid, branch_uuid])?;

        Ok(ReadOutput {
            user_roles,
            is_new_uuid_used,
            is_account_uuid_exist,
            is_company_branch_exist,
            is_account_uuid_with_company_branch_used,
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
                input.belong_to_account.to_string(),
                input.belong_to_company_branch.to_string(),
                input.outflow_type.as_str(),
                input.inflow_type.as_str(),
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
        test_query_helper_for_tables_schema(QUERY5).unwrap();
        test_query_helper_for_tables_schema(WRITE_QUERY).unwrap();
    }
}
