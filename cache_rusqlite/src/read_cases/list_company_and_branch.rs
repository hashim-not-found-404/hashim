use crate::utility::cache_adapter;
use crate::utility::utils::MyUuidConverter;
use crate::utility::utils::MyUuidConverter1;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types;
use my_core::accounting_domain::utility::types::DatabaseRead;
use my_core::utility::traits;
use rusqlite::params;
use std::str::FromStr;

const QUERY1: &str = "
    SELECT c.rowid, c.name, c.currency, acf.role
    FROM access_control_for_company acf
    JOIN company c ON acf.data_group = c.rowid
    WHERE acf.user_ = ?1";

const QUERY2: &str = "
    SELECT cb.rowid, cb.name, cb.currency, cb.company_belong, acfb.role
    FROM access_control_for_company_branch acfb
    JOIN company_branch cb ON acfb.data_group = cb.rowid
    WHERE acfb.user_ = ?1";

pub struct S;

impl cases::list_company_and_branch::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = cache_adapter::S;
    type Error = traits::DynamicError;
    type Input = cases::list_company_and_branch::ReadInput;
    type Output = cases::list_company_and_branch::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        use std::collections::HashMap;
        use types::Role;

        let company_query = QUERY1;
        let mut stmt = db.tables_db.prepare(company_query).unwrap();
        let company_rows = stmt
            .query_map(params![read_input.user_uuid.to_string()], |row| {
                let uuid: String = row.get(0).unwrap();
                let name: String = row.get(1).unwrap();
                let currency: String = row.get(2).unwrap();
                let role: Option<String> = row.get(3).unwrap();
                Ok((uuid, name, currency, role))
            })
            .unwrap();

        let mut company_map: HashMap<String, (String, String, Vec<Role>)> = HashMap::new();
        for row in company_rows {
            let (uuid, name, currency, role_opt) = row.unwrap();
            if let Some(role_str) = role_opt {
                let role = Role::from_str(&role_str).unwrap();
                company_map
                    .entry(uuid)
                    .or_insert_with(|| (name, currency, Vec::new()))
                    .2
                    .push(role);
            }
        }

        let branch_query = QUERY2;
        let mut stmt = db.tables_db.prepare(branch_query).unwrap();
        let branch_rows = stmt
            .query_map(params![read_input.user_uuid.to_string()], |row| {
                let branch_uuid: String = row.get(0).unwrap();
                let branch_name: String = row.get(1).unwrap();
                let branch_currency: String = row.get(2).unwrap();
                let company_belong: String = row.get(3).unwrap();
                let role: Option<String> = row.get(4).unwrap();
                Ok((branch_uuid, branch_name, branch_currency, company_belong, role))
            })
            .unwrap();

        struct BranchAccumulator {
            branch_uuid:     String,
            branch_name:     String,
            branch_currency: String,
            company_belong:  String,
            roles:           Vec<Role>,
        }
        let mut branch_map: HashMap<String, BranchAccumulator> = HashMap::new();
        for row in branch_rows {
            let (branch_uuid, branch_name, branch_currency, company_belong, role_opt) =
                row.unwrap();
            let entry = branch_map.entry(branch_uuid.clone()).or_insert_with(|| {
                BranchAccumulator {
                    branch_uuid:     branch_uuid.clone(),
                    branch_name:     branch_name.clone(),
                    branch_currency: branch_currency.clone(),
                    company_belong:  company_belong.clone(),
                    roles:           Vec::new(),
                }
            });
            if let Some(role_str) = role_opt {
                let role = Role::from_str(&role_str).unwrap();
                entry.roles.push(role);
            }
        }

        let mut result = Vec::new();
        for (company_uuid_str, (company_name, company_currency_str, company_roles)) in company_map {
            let company_uuid = company_uuid_str.clone().to_uuid();
            let company_currency = types::Currency::from_str(&company_currency_str).unwrap();

            let branches: Vec<cases::list_company_and_branch::AllBranchesThatUserInWithRoles> =
                branch_map
                    .iter()
                    .filter(|(_, info)| info.company_belong == company_uuid_str)
                    .map(|(_, info)| {
                        let branch_uuid = info.branch_uuid.clone().to_uuid();
                        let branch_currency =
                            types::Currency::from_str(&info.branch_currency).unwrap();
                        cases::list_company_and_branch::AllBranchesThatUserInWithRoles {
                            branch_uuid,
                            branch_name: info.branch_name.clone(),
                            branch_currancy: branch_currency,
                            user_roles: info.roles.clone(),
                        }
                    })
                    .collect::<Vec<_>>();

            result.push(cases::list_company_and_branch::AllCompaniesThatUserInWithRoles {
                company_uuid,
                company_name,
                company_currancy: company_currency,
                user_roles: company_roles,
                branches,
            });
        }

        Ok(cases::list_company_and_branch::ReadOutput {
            data: result,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::test_helper::test_query_helper_for_tables_schema;

    #[test]
    fn test_query_string_directly() {
        test_query_helper_for_tables_schema(QUERY1).unwrap();
        test_query_helper_for_tables_schema(QUERY2).unwrap();
    }
}
