use crate::domain::BranchInfo;
use crate::domain::CompanyWithBranches;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use database::db_client;
use database::utils::MyUuidConverter;
use database::utils::MyUuidConverter1;
use kernel::types::Currency;
use kernel::types::DatabaseRead;
use kernel::types::Role;
use serde::Deserialize;
use std::str::FromStr;
use utility::types::LogError;
use uuid::Uuid;

const READ_QUERY: &str = r#"
    WITH user_companies AS (
        SELECT
            c.rowid AS company_uuid,
            c.name AS company_name,
            c.currency AS company_currency,
            acf.role AS role
        FROM accounting_app.access_control_for_company acf
        JOIN accounting_app.company c ON acf.data_group = c.rowid
        WHERE acf.user_ = $1
    ),
    company_roles AS (
        SELECT
            company_uuid,
            jsonb_agg(DISTINCT role) AS roles
        FROM user_companies
        GROUP BY company_uuid
    ),
    visible_branches AS (
        SELECT
            cb.rowid AS branch_uuid,
            cb.name AS branch_name,
            cb.company_belong AS company_uuid
        FROM accounting_app.company_branch cb
        WHERE cb.company_belong IN (SELECT company_uuid FROM user_companies)
           OR cb.rowid IN (
               SELECT data_group FROM accounting_app.access_control_for_company_branch
               WHERE user_ = $1
           )
    ),
    branch_roles AS (
        SELECT
            data_group AS branch_uuid,
            jsonb_agg(DISTINCT role) AS roles
        FROM accounting_app.access_control_for_company_branch
        WHERE user_ = $1
        GROUP BY data_group
    )
    SELECT
        cr.company_uuid::text,
        c.name AS company_name,
        c.currency AS company_currency,
        cr.roles AS company_roles,
        COALESCE(
            jsonb_agg(
                jsonb_build_object(
                    'uuid', b.branch_uuid::text,
                    'name', b.branch_name,
                    'roles', COALESCE(br.roles, '[]'::jsonb)
                )
                ORDER BY b.branch_name
            ) FILTER (WHERE b.branch_uuid IS NOT NULL),
            '[]'::jsonb
        ) AS branches
    FROM company_roles cr
    JOIN accounting_app.company c ON c.rowid = cr.company_uuid
    LEFT JOIN visible_branches b ON b.company_uuid = cr.company_uuid
    LEFT JOIN branch_roles br ON br.branch_uuid = b.branch_uuid
    GROUP BY cr.company_uuid, c.name, c.currency, cr.roles
    ORDER BY c.name
"#;

pub struct DataBaseOp;

impl DatabaseRead for DataBaseOp {
    type Db<'a> = db_client::S;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db<'_>, input: &Self::Input) -> Result<Self::Output> {
        let stmt = db.client.prepare_cached(READ_QUERY).await.log()?;
        let rows = db
            .client
            .query(&stmt, &[&input.user_uuid.to_externel_uuid()])
            .await
            .log()?;

        #[derive(Debug, Deserialize)]
        struct BranchJson {
            uuid: String,
            name: String,
            roles: Vec<Role>,
        }

        let mut companies = Vec::with_capacity(rows.len());
        for row in rows {
            let company_uuid_str: String = row.try_get(0).log()?;
            let company_name: String = row.try_get(1).log()?;
            let company_currency_str: String = row.try_get(2).log()?;
            let company_roles_json: serde_json::Value = row.try_get(3).log()?;
            let branches_json: serde_json::Value = row.try_get(4).log()?;

            let company_uuid = Uuid::parse_str(&company_uuid_str).log()?.to_uuid();
            let currency = Currency::from_str(&company_currency_str).log()?;
            let roles: Vec<Role> = serde_json::from_value(company_roles_json).log()?;

            let raw_branches: Vec<BranchJson> = serde_json::from_value(branches_json).log()?;
            let branches = raw_branches
                .into_iter()
                .map(|b| {
                    let uuid = Uuid::parse_str(&b.uuid).log()?.to_uuid();
                    Ok(BranchInfo {
                        uuid: uuid.into(),
                        name: b.name,
                        roles: b.roles,
                    })
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()
                .log()?;

            companies.push(CompanyWithBranches {
                uuid: company_uuid.into(),
                name: company_name,
                currency,
                roles,
                branches,
            });
        }

        Ok(ReadOutput { companies })
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
