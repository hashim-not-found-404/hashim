use crate::utility::cache_adapter;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types::DatabaseRead;
use my_core::utility::traits;
use rusqlite::params;

const QUERY: &str = "SELECT
    EXISTS(SELECT 1 FROM user WHERE rowid = ?1),
    EXISTS(SELECT 1 FROM user WHERE id = ?2)";

pub struct S;

impl cases::sign_up::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = cache_adapter::S;
    type Error = traits::DynamicError;
    type ReadInput = cases::sign_up::ReadInput;
    type ReadOutput = cases::sign_up::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &Self::ReadInput,
    ) -> Result<Self::ReadOutput, Self::Error> {
        let query = QUERY;

        let a = db
            .tables_db
            .query_one(query, params![read_input.new_uuid.to_string(), read_input.user_id], |row| {
                Ok(cases::sign_up::ReadOutput {
                    is_new_uuid_exist: row.get(0).unwrap(),
                    is_user_id_exist:  row.get(1).unwrap(),
                })
            })
            .unwrap();

        Ok(a)
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
