use crate::utility::db_client;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types::DatabaseRead;
use my_core::accounting_domain::utility::types::{self};
use my_core::utility::traits;
use my_core::utility::utils::LogError;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

const READ_QUERY: &str = "
    WITH user_companies AS (
        SELECT
            c.rowid as company_uuid,
            c.name as company_name,
            c.currency as company_currency,
            acf.role as user_role
        FROM accounting_app.access_control_for_company acf
        JOIN accounting_app.company c ON acf.data_group = c.rowid
        WHERE acf.user_ = $1
    ),
    company_branches AS (
        SELECT
            cb.company_belong,
            json_agg(
                json_build_object(
                    'uuid', cb.rowid::text,
                    'name', cb.name,
                    'currency', cb.currency
                ) ORDER BY cb.name
            ) as branches
        FROM accounting_app.company_branch cb
        WHERE cb.company_belong IN (SELECT company_uuid FROM user_companies)
        GROUP BY cb.company_belong
    )
    SELECT
        uc.company_uuid::text,       -- cast to text to match JSON representation
        uc.company_name,
        uc.company_currency,
        uc.user_role,
        COALESCE(cb.branches, '[]'::json) as branches
    FROM user_companies uc
    LEFT JOIN company_branches cb ON uc.company_uuid = cb.company_belong
";

pub struct S;

impl cases::list_company_and_branch::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = db_client::S;
    type Error = traits::DynamicError;
    type Input = cases::list_company_and_branch::ReadInput;
    type Output = cases::list_company_and_branch::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        let rows =
            db.client.query(READ_QUERY, &[&read_input.user_uuid.to_externel_uuid()]).await.log()?;

        #[derive(Debug, Deserialize)]
        struct BranchJson {
            uuid:     String,
            name:     String,
            currency: String,
        }

        struct CompanyAgg {
            name:     String,
            currency: types::Currency,
            roles:    Vec<types::Role>,
            branches: Vec<cases::list_company_and_branch::AllBranchesThatUserInWithRoles>,
        }

        let mut company_map: HashMap<types::UuidType, CompanyAgg> = HashMap::new();

        for row in rows {
            let company_uuid_str: String = row.try_get(0).log()?;
            let company_uuid_parsed = Uuid::parse_str(&company_uuid_str).log()?;
            let company_uuid = types::UuidType(company_uuid_parsed.into_bytes());

            let company_name: String = row.try_get(1).log()?;
            let company_currency_str: String = row.try_get(2).log()?;
            let user_role_str: String = row.try_get(3).log()?;
            let branches_json: serde_json::Value = row.try_get(4).log()?;

            let company_currency = types::Currency::from_str(&company_currency_str).log()?;
            let role = types::Role::from_str(&user_role_str).log()?;

            let branches: Vec<BranchJson> = serde_json::from_value(branches_json).log()?;
            let branch_entries: Vec<
                cases::list_company_and_branch::AllBranchesThatUserInWithRoles,
            > = branches
                .into_iter()
                .map(|bj| {
                    let uuid = Uuid::parse_str(&bj.uuid).log()?;
                    let branch_uuid = types::UuidType(uuid.into_bytes());
                    let branch_currency = types::Currency::from_str(&bj.currency).log()?;
                    Ok(cases::list_company_and_branch::AllBranchesThatUserInWithRoles {
                        branch_uuid,
                        branch_name: bj.name,
                        branch_currancy: branch_currency,
                        user_roles: Vec::new(),
                    })
                })
                .collect::<Result<Vec<_>, traits::DynamicError>>()
                .log()?;

            let entry = company_map.entry(company_uuid).or_insert_with(|| {
                CompanyAgg {
                    name:     company_name.clone(),
                    currency: company_currency.clone(),
                    roles:    Vec::new(),
                    branches: Vec::new(),
                }
            });

            entry.name = company_name;
            entry.currency = company_currency;
            if !entry.roles.contains(&role) {
                entry.roles.push(role);
            }
            entry.branches = branch_entries;
        }

        let data = company_map
            .into_iter()
            .map(|(company_uuid, agg)| {
                cases::list_company_and_branch::AllCompaniesThatUserInWithRoles {
                    company_uuid,
                    company_name: agg.name,
                    company_currancy: agg.currency,
                    user_roles: agg.roles,
                    branches: agg.branches,
                }
            })
            .collect();

        Ok(cases::list_company_and_branch::ReadOutput {
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::test_helper::test_query_helper;

    #[tokio::test]
    async fn test_query_string_directly() {
        test_query_helper(READ_QUERY).await.unwrap();
    }
}
