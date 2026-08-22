use crate::utility::cache_adapter;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types::DatabaseRead;
use my_core::accounting_domain::utility::types::Role;
use my_core::utility::traits::DynamicError;
use rusqlite::params;
use std::str::FromStr;

const QUERY1: &str =
    "SELECT role FROM access_control_for_company WHERE data_group = ?1 AND user_ = ?2";
const QUERY2: &str = "SELECT 1 FROM company WHERE rowid = ?1";
const QUERY3: &str = "SELECT 1 FROM company_branch WHERE company_belong = ?1 AND name = ?2";

pub struct S;

impl cases::create_company_branch::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = cache_adapter::S;
    type Input = cases::create_company_branch::ReadInput;
    type Output = cases::create_company_branch::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        input: &Self::Input,
    ) -> Result<Self::Output, DynamicError> {
        let mut stmt = db.tables_db.prepare(QUERY1).unwrap();

        let roles_iter = stmt
            .query_map(
                params![input.company_belong.to_string(), input.user_uuid.to_string()],
                |row| {
                    let role_str: String = row.get(0).unwrap();
                    let role = Role::from_str(role_str.as_str()).unwrap();
                    Ok(role)
                },
            )
            .unwrap();

        let mut roles = Vec::new();
        for role in roles_iter {
            roles.push(role.unwrap());
        }

        let mut stmt = db.tables_db.prepare(QUERY2).unwrap();
        let company_exists = stmt.exists(params![input.company_belong.to_string()]).unwrap();

        let mut stmt = db.tables_db.prepare(QUERY3).unwrap();
        let branch_name_used =
            stmt.exists(params![input.company_belong.to_string(), input.branch_name]).unwrap();

        let a = cases::create_company_branch::ReadOutput {
            user_roles:          roles,
            is_new_uuid_used:    false,
            is_company_exist:    company_exists,
            is_branch_name_used: branch_name_used,
        };

        Ok(a)
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
    }
}
