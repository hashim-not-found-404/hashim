use crate::utility::cache_adapter;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::utility::traits;
use rusqlite::params;

pub struct S;

impl cases::sign_up::DatabaseRead for S {
    type Db<'a> = cache_adapter::S;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &cases::sign_up::ReadInput,
    ) -> Result<cases::sign_up::ReadOutput, traits::DynamicError> {
        let query = "
            SELECT
                EXISTS(SELECT 1 FROM user WHERE rowid = ?1),
                EXISTS(SELECT 1 FROM user WHERE id = ?2)
        ";

        let a = db
            .db
            .query_one(
                query,
                params![read_input.new_uuid.to_string(), read_input.user_id],
                |row| {
                    Ok(cases::sign_up::ReadOutput {
                        is_new_uuid_exist: row.get(0).unwrap(),
                        is_user_id_exist: row.get(1).unwrap(),
                    })
                },
            )
            .unwrap();

        Ok(a)
    }
}
