use crate::utility::cache_adapter;
use crate::utility::utils::MyUuidConverter1;
use my_core::accounting_domain::cases;
use my_core::utility::traits;
use rusqlite::params;

pub struct S;

impl cases::sign_in::DatabaseRead for S {
    type Db<'a> = cache_adapter::S;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::sign_in::ReadInput,
    ) -> Result<cases::sign_in::ReadOutput, traits::DynamicError> {
        let query = "SELECT rowid, name, jwt FROM user WHERE id = ?1;";

        let a = db
            .db
            .query_row(query, params![read_input.user_id], |row| {
                let user_uuid_str: String = row.get(0).unwrap();
                let user_name: Option<String> = row.get(1).unwrap();
                let jwt: Option<String> = row.get(2).unwrap();

                let a = Some((
                    user_uuid_str.to_uuid(),
                    jwt.unwrap_or_default(), // true if JWT exists
                    user_name,
                ));

                Ok(cases::sign_in::ReadOutput {
                    user_rowid_and_password_hash_and_name: a,
                })
            })
            .unwrap();

        Ok(a)
    }
}
