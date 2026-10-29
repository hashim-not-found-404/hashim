use crate::domain::Data;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use cache::cache_adapter;
use cache::utils::MyUuidConverter;
use cache::utils::MyUuidConverter1;
use kernel::types::DatabaseRead;
use kernel::types::DatabaseWrite;
use kernel::types::Role;
use rusqlite::params;
use serde::Deserialize;
use serde_json::Value;
use serde_json::from_str;
use std::str::FromStr;

const READ_QUERY: &str = "
    WITH
    roles_cte AS (
        SELECT COALESCE(json_group_array(role), '[]') AS roles
        FROM access_control_for_company
        WHERE data_group = ?1 AND user_ = ?2
    ),
    journals_cte AS (
        SELECT rowid AS row_uuid, name, 1 AS is_it_contain_quantity
        FROM journal_with_quantity
        WHERE company_belong = ?1
        UNION ALL
        SELECT rowid, name, 0
        FROM journal_without_quantity
        WHERE company_belong = ?1
    )
    SELECT
        (SELECT roles FROM roles_cte) AS roles,
        COALESCE(
            (SELECT json_group_array(json_object(
                'row_uuid', row_uuid,
                'name', name,
                'is_it_contain_quantity', is_it_contain_quantity
            )) FROM journals_cte),
            '[]'
        ) AS data
";

const DELETE_WITH_QUANTITY_QUERY: &str =
    "DELETE FROM journal_with_quantity WHERE company_belong = ?1";
const DELETE_WITHOUT_QUANTITY_QUERY: &str =
    "DELETE FROM journal_without_quantity WHERE company_belong = ?1";

const WRITE_WITH_QUANTITY_QUERY: &str = "
    INSERT OR REPLACE INTO journal_with_quantity (rowid, company_belong, name)
    VALUES (?1, ?2, ?3)
";
const WRITE_WITHOUT_QUANTITY_QUERY: &str = "
    INSERT OR REPLACE INTO journal_without_quantity (rowid, company_belong, name)
    VALUES (?1, ?2, ?3)
";

#[derive(Deserialize)]
struct RawData {
    row_uuid: String,
    name: String,
    is_it_contain_quantity: bool,
}

pub struct CacheOp;

impl DatabaseRead for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db<'_>, input: &Self::Input) -> Result<Self::Output> {
        let mut stmt = db.tables_db.prepare(READ_QUERY)?;
        let (roles_str, data_str): (String, String) = stmt.query_row(
            params![input.company_uuid.to_string(), input.user_uuid.to_string(),],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let roles_value: Value = from_str(&roles_str).unwrap_or(Value::Array(vec![]));
        let user_roles: Vec<Role> = roles_value
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| Role::from_str(s).ok())
                    .collect()
            })
            .unwrap_or_default();

        let data_value: Value = from_str(&data_str).unwrap_or(Value::Array(vec![]));
        let raw: Vec<RawData> = serde_json::from_value(data_value)?;
        let data = raw
            .into_iter()
            .map(|r| -> Result<Data> {
                Ok(Data {
                    row_uuid: r.row_uuid.to_uuid()?.into(),
                    name: r.name,
                    is_it_contain_quantity: r.is_it_contain_quantity,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(ReadOutput { user_roles, data })
    }
}

impl DatabaseWrite for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<()> {
        let company_uuid = input.company_uuid.to_string();

        txn.tables_db
            .execute(DELETE_WITH_QUANTITY_QUERY, params![&company_uuid])?;
        txn.tables_db
            .execute(DELETE_WITHOUT_QUANTITY_QUERY, params![&company_uuid])?;

        for data in &input.data {
            let query = if data.is_it_contain_quantity {
                WRITE_WITH_QUANTITY_QUERY
            } else {
                WRITE_WITHOUT_QUANTITY_QUERY
            };
            txn.tables_db.execute(
                query,
                params![data.row_uuid.to_string(), &company_uuid, &data.name],
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cache::test_helper::test_query_helper_for_tables_schema;

    #[test]
    fn test_query_string_directly() {
        test_query_helper_for_tables_schema(READ_QUERY).unwrap();
        test_query_helper_for_tables_schema(DELETE_WITH_QUANTITY_QUERY).unwrap();
        test_query_helper_for_tables_schema(DELETE_WITHOUT_QUANTITY_QUERY).unwrap();
        test_query_helper_for_tables_schema(WRITE_WITH_QUANTITY_QUERY).unwrap();
        test_query_helper_for_tables_schema(WRITE_WITHOUT_QUANTITY_QUERY).unwrap();
    }
}
