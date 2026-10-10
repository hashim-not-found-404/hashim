use crate::domain::BranchInfo;
use crate::domain::CompanyWithBranches;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use anyhow::Result;
use cache::cache_adapter;
use cache::utils::MyUuidConverter;
use cache::utils::MyUuidConverter1;
use kernel::types::Currency;
use kernel::types::DatabaseRead;
use kernel::types::DatabaseWrite;
use rusqlite::params;
use std::collections::HashMap;
use std::str::FromStr;

const READ_COMPANIES_QUERY: &str = "
    SELECT c.rowid, c.name, c.currency
    FROM company c
    WHERE EXISTS (
        SELECT 1 FROM access_control_for_company
        WHERE data_group = c.rowid AND user_ = ?1
    )
    ORDER BY c.name
";

const READ_BRANCHES_QUERY: &str = "
    SELECT b.rowid, b.name, b.company_belong
    FROM company_branch b
    WHERE EXISTS (
        SELECT 1 FROM access_control_for_company
        WHERE data_group = b.company_belong AND user_ = ?1
    )
    OR EXISTS (
        SELECT 1 FROM access_control_for_company_branch
        WHERE data_group = b.rowid AND user_ = ?1
    )
    ORDER BY b.name
";

pub struct CacheOp;

impl DatabaseRead for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db<'_>, input: &Self::Input) -> Result<Self::Output> {
        let user_uuid = input.user_uuid.to_string();

        let mut stmt = db.tables_db.prepare(READ_COMPANIES_QUERY)?;
        let rows = stmt.query_map(params![&user_uuid], |row| {
            let uuid_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let currency_str: String = row.get(2)?;
            Ok((uuid_str, name, currency_str))
        })?;

        let mut companies: Vec<CompanyWithBranches> = Vec::new();
        let mut index_by_company: HashMap<String, usize> = HashMap::new();
        for row in rows {
            let (uuid_str, name, currency_str) = row?;
            let uuid = uuid_str.clone().to_uuid()?;
            let currency = Currency::from_str(&currency_str).unwrap_or_default();
            index_by_company.insert(uuid_str, companies.len());
            companies.push(CompanyWithBranches {
                uuid: uuid.into(),
                name,
                currency,
                roles: Vec::new(),
                branches: Vec::new(),
            });
        }

        let mut stmt = db.tables_db.prepare(READ_BRANCHES_QUERY)?;
        let rows = stmt.query_map(params![&user_uuid], |row| {
            let uuid_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let company_belong: String = row.get(2)?;
            Ok((uuid_str, name, company_belong))
        })?;

        for row in rows {
            let (uuid_str, name, company_belong) = row?;
            let Some(&idx) = index_by_company.get(&company_belong) else {
                continue;
            };
            let uuid = uuid_str.to_uuid()?;
            companies[idx].branches.push(BranchInfo {
                uuid: uuid.into(),
                name,
                roles: Vec::new(),
            });
        }

        Ok(ReadOutput { companies })
    }
}

const WRITE_COMPANY_QUERY: &str =
    "INSERT OR REPLACE INTO company (rowid, name, currency) VALUES (?1, ?2, ?3)";

const WRITE_BRANCH_QUERY: &str = "
    INSERT OR REPLACE INTO company_branch (rowid, company_belong, name)
    VALUES (?1, ?2, ?3)
";

const WRITE_ACCESS_COMPANY_QUERY: &str = "
    INSERT OR REPLACE INTO access_control_for_company (rowid, data_group, user_, role)
    VALUES (?1, ?2, ?3, ?4)
";

const WRITE_ACCESS_BRANCH_QUERY: &str = "
    INSERT OR REPLACE INTO access_control_for_company_branch (rowid, data_group, user_, role)
    VALUES (?1, ?2, ?3, ?4)
";

impl DatabaseWrite for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<()> {
        let user_uuid = input.user_uuid.to_string();

        for company in &input.companies {
            let company_uuid = company.uuid.to_string();

            txn.tables_db.execute(
                WRITE_COMPANY_QUERY,
                params![&company_uuid, &company.name, company.currency.as_str()],
            )?;

            for role in &company.roles {
                let access_rowid = format!("{}_{}", company_uuid, role.as_str());
                txn.tables_db.execute(
                    WRITE_ACCESS_COMPANY_QUERY,
                    params![&access_rowid, &company_uuid, &user_uuid, role.as_str()],
                )?;
            }

            for branch in &company.branches {
                let branch_uuid = branch.uuid.to_string();

                txn.tables_db.execute(
                    WRITE_BRANCH_QUERY,
                    params![&branch_uuid, &company_uuid, &branch.name],
                )?;

                for role in &branch.roles {
                    let access_rowid = format!("{}_{}", branch_uuid, role.as_str());
                    txn.tables_db.execute(
                        WRITE_ACCESS_BRANCH_QUERY,
                        params![&access_rowid, &branch_uuid, &user_uuid, role.as_str()],
                    )?;
                }
            }
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
        test_query_helper_for_tables_schema(READ_COMPANIES_QUERY).unwrap();
        test_query_helper_for_tables_schema(READ_BRANCHES_QUERY).unwrap();
        test_query_helper_for_tables_schema(WRITE_COMPANY_QUERY).unwrap();
        test_query_helper_for_tables_schema(WRITE_BRANCH_QUERY).unwrap();
        test_query_helper_for_tables_schema(WRITE_ACCESS_COMPANY_QUERY).unwrap();
        test_query_helper_for_tables_schema(WRITE_ACCESS_BRANCH_QUERY).unwrap();
    }
}
