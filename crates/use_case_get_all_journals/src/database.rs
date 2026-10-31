use crate::domain::JournalInfo;
use crate::domain::JournalUuid;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use std::collections::HashMap;
use std::str::FromStr;
use use_case_prelude_database::*;

const READ_QUERY: &str = "
    WITH
    roles_cte AS (
        SELECT COALESCE(
            jsonb_agg(jsonb_build_object(
                'company_uuid', data_group,
                'role',         role
            )),
            '[]'::jsonb
        ) AS roles
        FROM accounting_app.access_control_for_company
        WHERE user_ = $1
    ),
    user_companies AS (
        SELECT DISTINCT data_group
        FROM accounting_app.access_control_for_company
        WHERE user_ = $1
    ),
    journals_cte AS (
        SELECT
            rowid::text          AS row_uuid,
            company_belong       AS company_uuid,
            name,
            true                 AS is_it_contain_quantity
        FROM accounting_app.journal_with_quantity
        WHERE company_belong IN (SELECT data_group FROM user_companies)

        UNION ALL

        SELECT
            rowid::text,
            company_belong,
            name,
            false
        FROM accounting_app.journal_without_quantity
        WHERE company_belong IN (SELECT data_group FROM user_companies)
    )
    SELECT
        (SELECT roles FROM roles_cte) AS roles,
        COALESCE(
            (SELECT jsonb_agg(jsonb_build_object(
                'row_uuid',               row_uuid,
                'company_uuid',           company_uuid,
                'name',                   name,
                'is_it_contain_quantity', is_it_contain_quantity
            )) FROM journals_cte),
            '[]'::jsonb
        ) AS data
";

#[derive(Deserialize)]
struct RawRole {
    company_uuid: String,
    role: String,
}

#[derive(Deserialize)]
struct RawData {
    row_uuid: String,
    company_uuid: String,
    name: String,
    is_it_contain_quantity: bool,
}

pub struct DataBaseOp;

impl DatabaseRead for DataBaseOp {
    type Db<'a> = DbClientAdapter;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db<'_>, input: &Self::Input) -> Result<Self::Output> {
        let row = db
            .client
            .query_one(READ_QUERY, &[&input.user_uuid.to_externel_uuid()])
            .await
            .log()?;

        // ---- roles (flat list -> HashMap<company, Vec<Role>>) ----
        let roles_value: Value = row.try_get(0).log()?;
        let raw_roles: Vec<RawRole> = from_value(roles_value).log()?;
        let mut user_roles: HashMap<CompanyUuid, Vec<Role>> = HashMap::new();
        for r in raw_roles {
            let role = match Role::from_str(&r.role) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let uuid_parsed = match Uuid::parse_str(&r.company_uuid) {
                Ok(u) => u,
                Err(_) => continue,
            };
            let company: CompanyUuid = UuidType::from(uuid_parsed.into_bytes()).into();
            user_roles.entry(company).or_default().push(role);
        }

        // ---- journals ----
        let data_value: Value = row.try_get(1).log()?;
        let raw: Vec<RawData> = from_value(data_value).log()?;
        let data = raw
            .into_iter()
            .map(|r| -> Result<JournalInfo> {
                let row_parsed = Uuid::parse_str(&r.row_uuid).log()?;
                let company_parsed = Uuid::parse_str(&r.company_uuid).log()?;

                let row_uuid: JournalUuid = UuidType::from(row_parsed.into_bytes()).into();
                let company_uuid: CompanyUuid = UuidType::from(company_parsed.into_bytes()).into();

                Ok(JournalInfo {
                    row_uuid,
                    company_uuid,
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

    #[tokio::test]
    async fn test_query_string_directly() {
        test_query_helper(READ_QUERY).await.unwrap();
    }
}
