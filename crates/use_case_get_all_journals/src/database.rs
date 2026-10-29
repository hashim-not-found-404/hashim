use crate::domain::Data;
use crate::domain::JournalUuid;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use database::db_client;
use database::utils::MyUuidConverter;
use kernel::new_types::UuidType;
use kernel::types::DatabaseRead;
use kernel::types::Role;
use serde::Deserialize;
use serde_json::Value;
use serde_json::from_value;
use std::str::FromStr;
use utility::types::LogError;
use uuid::Uuid;

const READ_QUERY: &str = "
    WITH
    roles_cte AS (
        SELECT COALESCE(jsonb_agg(role), '[]'::jsonb) AS roles
        FROM accounting_app.access_control_for_company
        WHERE data_group = $1 AND user_ = $2
    ),
    journals_cte AS (
        SELECT rowid::text AS row_uuid, name, true AS is_it_contain_quantity
        FROM accounting_app.journal_with_quantity
        WHERE company_belong = $1
        UNION ALL
        SELECT rowid::text, name, false
        FROM accounting_app.journal_without_quantity
        WHERE company_belong = $1
    )
    SELECT
        (SELECT roles FROM roles_cte) AS roles,
        COALESCE(
            (SELECT jsonb_agg(jsonb_build_object(
                'row_uuid', row_uuid,
                'name', name,
                'is_it_contain_quantity', is_it_contain_quantity
            )) FROM journals_cte),
            '[]'::jsonb
        ) AS data
";

#[derive(Deserialize)]
struct RawData {
    row_uuid: String,
    name: String,
    is_it_contain_quantity: bool,
}

pub struct DataBaseOp;

impl DatabaseRead for DataBaseOp {
    type Db<'a> = db_client::S;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db<'_>, input: &Self::Input) -> Result<Self::Output> {
        let row = db
            .client
            .query_one(
                READ_QUERY,
                &[
                    &input.company_uuid.to_externel_uuid(),
                    &input.user_uuid.to_externel_uuid(),
                ],
            )
            .await
            .log()?;

        let roles_value: Value = row.try_get(0).log()?;
        let user_roles: Vec<Role> = roles_value
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| Role::from_str(s).ok())
                    .collect()
            })
            .unwrap_or_default();

        let data_value: Value = row.try_get(1).log()?;
        let raw: Vec<RawData> = from_value(data_value).log()?;
        let data = raw
            .into_iter()
            .map(|r| -> Result<Data> {
                let uuid_parsed = Uuid::parse_str(&r.row_uuid).log()?;
                let row_uuid: JournalUuid = UuidType::from(uuid_parsed.into_bytes()).into();
                Ok(Data {
                    row_uuid,
                    name: r.name,
                    is_it_contain_quantity: r.is_it_contain_quantity,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(ReadOutput { user_roles, data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use database::test_helper::test_query_helper;

    #[tokio::test]
    async fn test_query_string_directly() {
        test_query_helper(READ_QUERY).await.unwrap();
    }
}
