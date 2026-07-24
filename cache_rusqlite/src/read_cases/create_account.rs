use crate::utility::cache_adapter;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types;
use my_core::utility::traits;
use rusqlite::params;
use std::str::FromStr;

pub struct S;

impl cases::create_account::DatabaseRead for S {
    type Db<'a> = cache_adapter::S;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::create_account::ReadInput,
    ) -> Result<cases::create_account::ReadOutput, traits::DynamicError> {
        // 1. User roles at the company
        let mut stmt = db
            .db
            .prepare(
                "SELECT role FROM access_control_for_company WHERE data_group = ?1 AND user_ = ?2",
            )
            .unwrap();
        let roles_iter = stmt
            .query_map(
                params![read_input.belong_to_company.to_string(), read_input.user_uuid.to_string()],
                |row| {
                    let role_str: String = row.get(0)?;
                    let role = types::Role::from_str(role_str.as_str()).unwrap();
                    Ok(role)
                },
            )
            .unwrap();
        let user_roles: Vec<types::Role> = roles_iter.map(|r| r.unwrap()).collect();

        // 2. Company exists
        let mut stmt = db.db.prepare("SELECT 1 FROM company WHERE rowid = ?1").unwrap();
        let is_company_uuid_exist =
            stmt.exists(params![read_input.belong_to_company.to_string()]).unwrap();

        // 3. New UUID already used
        let mut stmt = db.db.prepare("SELECT 1 FROM account WHERE rowid = ?1").unwrap();
        let is_new_uuid_used = stmt.exists(params![read_input.new_uuid.to_string()]).unwrap();

        // 4. Account name already used under the same company
        let mut stmt = db
            .db
            .prepare("SELECT 1 FROM account WHERE belong_to_company = ?1 AND name = ?2")
            .unwrap();
        let is_account_name_used = stmt
            .exists(params![read_input.belong_to_company.to_string(), &read_input.account_name])
            .unwrap();

        Ok(cases::create_account::ReadOutput {
            is_company_uuid_exist,
            is_new_uuid_used,
            user_roles,
            is_account_name_used,
        })
    }
}
