use crate::utility::cache_adapter;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types;
use my_core::utility::traits;
use rusqlite::params;
use std::str::FromStr;

pub struct S;

impl cases::create_company_branch::DatabaseRead for S {
    type Db<'a> = cache_adapter::S;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::create_company_branch::ReadInput,
    ) -> Result<cases::create_company_branch::ReadOutput, traits::DynamicError> {
        // 1. Get the user's roles in the company
        let mut stmt = db
            .db
            .prepare(
                "SELECT role FROM access_control_for_company WHERE data_group = ?1 AND user_ = ?2",
            )
            .unwrap();

        let roles_iter = stmt
            .query_map(
                params![read_input.company_belong.to_string(), read_input.user_uuid.to_string()],
                |row| {
                    let role_str: String = row.get(0)?;
                    let role = types::Role::from_str(role_str.as_str()).unwrap();
                    Ok(role)
                },
            )
            .unwrap();

        let mut roles = Vec::new();
        for role in roles_iter {
            roles.push(role.unwrap());
        }

        // 2. Check if the company exists
        let mut stmt = db.db.prepare("SELECT 1 FROM company WHERE rowid = ?1").unwrap();
        let company_exists = stmt.exists(params![read_input.company_belong.to_string()]).unwrap();

        // 3. Check if the branch name is already used under this company
        let mut stmt = db
            .db
            .prepare("SELECT 1 FROM company_branch WHERE company_belong = ?1 AND name = ?2")
            .unwrap();
        let branch_name_used = stmt
            .exists(params![read_input.company_belong.to_string(), read_input.branch_name])
            .unwrap();

        let a = cases::create_company_branch::ReadOutput {
            user_roles:          roles,
            is_new_uuid_used:    false,
            is_company_exist:    company_exists,
            is_branch_name_used: branch_name_used,
        };

        Ok(a)
    }
}
