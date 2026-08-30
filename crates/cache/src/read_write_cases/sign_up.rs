use crate::utility::cache_adapter;
use crate::utility::utils::MyUuidConverter;
use my_core::domain::use_cases;
use my_core::domain::utility::types::DatabaseRead;
use my_core::domain::utility::types::DatabaseWrite;
use my_core::utility::traits::DynamicError;
use rusqlite::params;

const QUERY: &str = "SELECT
    EXISTS(SELECT 1 FROM user WHERE rowid = ?1),
    EXISTS(SELECT 1 FROM user WHERE id = ?2)";

pub struct S;

impl use_cases::sign_up::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = cache_adapter::S;
    type Input = use_cases::sign_up::ReadInput;
    type Output = use_cases::sign_up::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        input: &Self::Input,
    ) -> Result<Self::Output, DynamicError> {
        let query = QUERY;

        let a = db
            .tables_db
            .query_one(query, params![input.new_uuid.to_string(), input.user_id], |row| {
                Ok(use_cases::sign_up::ReadOutput {
                    is_new_uuid_exist: row.get(0).unwrap(),
                    is_user_id_exist:  row.get(1).unwrap(),
                })
            })
            .unwrap();

        Ok(a)
    }
}

impl DatabaseWrite for S {
    type Db<'a> = cache_adapter::S;
    type Input = use_cases::sign_up::Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<(), DynamicError> {
        todo!()
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
