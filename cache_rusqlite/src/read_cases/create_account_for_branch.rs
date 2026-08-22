use crate::utility::cache_adapter;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types::DatabaseRead;
use my_core::accounting_domain::utility::types::{self};
use my_core::utility::traits::DynamicError;
use rusqlite::params;
use std::str::FromStr;

const QUERY1: &str = "SELECT role FROM access_control_for_company
     WHERE data_group = (SELECT company_belong FROM company_branch WHERE rowid = ?1)
     AND user_ = ?2";
const QUERY2: &str = "SELECT 1 FROM account_flow_type WHERE rowid = ?1";
const QUERY3: &str = "SELECT 1 FROM account WHERE rowid = ?1";
const QUERY4: &str = "SELECT 1 FROM company_branch WHERE rowid = ?1";
const QUERY5: &str = "SELECT 1 FROM account_flow_type WHERE account = ?1 AND company_branch = ?2";

pub struct S;

impl cases::create_account_for_branch::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = cache_adapter::S;
    type Input = cases::create_account_for_branch::ReadInput;
    type Output = cases::create_account_for_branch::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        input: &Self::Input,
    ) -> Result<Self::Output, DynamicError> {
        let branch_uuid = input.belong_to_company_branch.to_string();
        let user_uuid = input.user_uuid.to_string();
        let new_uuid = input.new_uuid.to_string();
        let account_uuid = input.belong_to_account.to_string();
        let mut stmt = db.tables_db.prepare(QUERY1).unwrap();
        let roles_iter = stmt
            .query_map(params![branch_uuid, user_uuid], |row| {
                let role_str: String = row.get(0).unwrap();
                let role = types::Role::from_str(&role_str).unwrap();
                Ok(role)
            })
            .unwrap();

        let mut user_roles = Vec::new();
        for role in roles_iter {
            user_roles.push(role.unwrap());
        }
        let mut stmt = db.tables_db.prepare(QUERY2).unwrap();
        let is_new_uuid_used = stmt.exists(params![new_uuid]).unwrap();

        let mut stmt = db.tables_db.prepare(QUERY3).unwrap();
        let is_account_uuid_exist = stmt.exists(params![account_uuid]).unwrap();

        let mut stmt = db.tables_db.prepare(QUERY4).unwrap();
        let is_company_branch_exist = stmt.exists(params![branch_uuid]).unwrap();

        let mut stmt = db.tables_db.prepare(QUERY5).unwrap();
        let is_account_uuid_with_company_branch_used =
            stmt.exists(params![account_uuid, branch_uuid]).unwrap();

        Ok(cases::create_account_for_branch::ReadOutput {
            user_roles,
            is_new_uuid_used,
            is_account_uuid_exist,
            is_company_branch_exist,
            is_account_uuid_with_company_branch_used,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::test_helper::test_query_helper_for_tables_schema;

    #[test]
    fn test_query_string_directly() {
        test_query_helper_for_tables_schema(QUERY1).unwrap();
        test_query_helper_for_tables_schema(QUERY2).unwrap();
        test_query_helper_for_tables_schema(QUERY3).unwrap();
        test_query_helper_for_tables_schema(QUERY4).unwrap();
        test_query_helper_for_tables_schema(QUERY5).unwrap();
    }
}
