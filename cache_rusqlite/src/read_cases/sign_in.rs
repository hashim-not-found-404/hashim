use crate::utility::cache_adapter;
use crate::utility::utils::MyUuidConverter1;
use my_core::domain::cases;
use my_core::domain::utility::types::DatabaseRead;
use my_core::utility::traits::DynamicError;
use rusqlite::OptionalExtension;
use rusqlite::params;

const QUERY: &str = "SELECT rowid, name, jwt FROM user WHERE id = ?1;";

pub struct S;

impl cases::sign_in::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = cache_adapter::S;
    type Input = cases::sign_in::ReadInput;
    type Output = cases::sign_in::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        input: &Self::Input,
    ) -> Result<Self::Output, DynamicError> {
        let query = QUERY;

        let a = db
            .tables_db
            .query_row(query, params![input.user_id], |row| {
                let user_uuid_str: String = row.get(0).unwrap();
                let user_name: Option<String> = row.get(1).unwrap();
                let jwt: Option<String> = row.get(2).unwrap();

                Ok((user_uuid_str.to_uuid().into(), jwt.unwrap_or_default(), user_name))
            })
            .optional()
            .unwrap();

        Ok(cases::sign_in::ReadOutput {
            user_rowid_and_password_hash_and_name: a,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::test_helper::test_query_helper_for_tables_schema;

    #[test]
    fn test_query_string_directly() {
        test_query_helper_for_tables_schema(QUERY).unwrap();
    }
}
