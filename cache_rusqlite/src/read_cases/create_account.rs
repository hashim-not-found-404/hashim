use crate::utility::cache_adapter;
use crate::utility::utils::MyUuidConverter;
use my_core::domain::cases;
use my_core::domain::utility::types::DatabaseRead;
use my_core::domain::utility::types::Role;
use my_core::utility::traits::DynamicError;
use rusqlite::params;
use std::str::FromStr;

const QUERY1: &str =
    "SELECT role FROM access_control_for_company WHERE data_group = ?1 AND user_ = ?2";
const QUERY2: &str = "SELECT 1 FROM company WHERE rowid = ?1";
const QUERY3: &str = "SELECT 1 FROM account WHERE rowid = ?1";
const QUERY4: &str = "SELECT 1 FROM account WHERE belong_to_company = ?1 AND name = ?2";

pub struct S;
impl cases::create_account::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = cache_adapter::S;
    type Input = cases::create_account::ReadInput;
    type Output = cases::create_account::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        input: &Self::Input,
    ) -> Result<Self::Output, DynamicError> {
        let mut stmt = db.tables_db.prepare(QUERY1).unwrap();
        let roles_iter = stmt
            .query_map(
                params![input.belong_to_company.to_string(), input.user_uuid.to_string()],
                |row| {
                    let role_str: String = row.get(0).unwrap();
                    let role = Role::from_str(role_str.as_str()).unwrap();
                    Ok(role)
                },
            )
            .unwrap();
        let user_roles: Vec<Role> = roles_iter.map(|r| r.unwrap()).collect();
        let mut stmt = db.tables_db.prepare(QUERY2).unwrap();
        let is_company_uuid_exist =
            stmt.exists(params![input.belong_to_company.to_string()]).unwrap();
        let mut stmt = db.tables_db.prepare(QUERY3).unwrap();
        let is_new_uuid_used = stmt.exists(params![input.new_uuid.to_string()]).unwrap();
        let mut stmt = db.tables_db.prepare(QUERY4).unwrap();
        let is_account_name_used =
            stmt.exists(params![input.belong_to_company.to_string(), &input.account_name]).unwrap();

        Ok(cases::create_account::ReadOutput {
            is_company_uuid_exist,
            is_new_uuid_used,
            user_roles,
            is_account_name_used,
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
    }
}
