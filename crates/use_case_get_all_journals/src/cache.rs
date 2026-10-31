use crate::domain::JournalInfo;
use crate::domain::JournalUuid;
use crate::domain::Ok;
use crate::domain::ReadInput;
use crate::domain::ReadOutput;
use std::collections::HashMap;
use std::collections::HashSet;
use std::str::FromStr;
use use_case_prelude_cache::*;

const READ_QUERY: &str = "
    WITH
    roles_cte AS (
        SELECT COALESCE(
            json_group_array(json_object(
                'company_uuid', data_group,
                'role',         role
            )),
            '[]'
        ) AS roles
        FROM access_control_for_company
        WHERE user_ = ?1
    ),
    user_companies AS (
        SELECT DISTINCT data_group
        FROM access_control_for_company
        WHERE user_ = ?1
    ),
    journals_cte AS (
        SELECT
            rowid           AS row_uuid,
            company_belong  AS company_uuid,
            name,
            1               AS is_it_contain_quantity
        FROM journal_with_quantity
        WHERE company_belong IN (SELECT data_group FROM user_companies)

        UNION ALL

        SELECT
            rowid,
            company_belong,
            name,
            0
        FROM journal_without_quantity
        WHERE company_belong IN (SELECT data_group FROM user_companies)
    )
    SELECT
        (SELECT roles FROM roles_cte) AS roles,
        COALESCE(
            (SELECT json_group_array(json_object(
                'row_uuid',               row_uuid,
                'company_uuid',           company_uuid,
                'name',                   name,
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
struct RawRole {
    company_uuid: String,
    role: String,
}

#[derive(Deserialize)]
struct RawData {
    row_uuid: String,
    company_uuid: String,
    name: String,
    // SQLite JSON emits 0/1 integers, not booleans.
    is_it_contain_quantity: i64,
}

pub struct CacheOp;

impl DatabaseRead for CacheOp {
    type Db<'a> = CacheAdapter;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db<'_>, input: &Self::Input) -> Result<Self::Output> {
        let mut stmt = db.tables_db.prepare(READ_QUERY)?;
        let (roles_str, data_str): (String, String) = stmt
            .query_row(params![input.user_uuid.to_string()], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?;

        // ---- roles (flat list -> HashMap<company, Vec<Role>>) ----
        let roles_value: Value = from_str(&roles_str).unwrap_or(Value::Array(vec![]));
        let raw_roles: Vec<RawRole> = from_value(roles_value).unwrap_or_default();
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
        let data_value: Value = from_str(&data_str).unwrap_or(Value::Array(vec![]));
        let raw: Vec<RawData> = from_value(data_value)?;
        let data = raw
            .into_iter()
            .map(|r| -> Result<JournalInfo> {
                let company_parsed = Uuid::parse_str(&r.company_uuid)?;
                let row_uuid: JournalUuid = r.row_uuid.to_uuid()?.into();
                let company_uuid: CompanyUuid = UuidType::from(company_parsed.into_bytes()).into();

                Ok(JournalInfo {
                    row_uuid,
                    company_uuid,
                    name: r.name,
                    is_it_contain_quantity: r.is_it_contain_quantity != 0,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(ReadOutput { user_roles, data })
    }
}

impl DatabaseWrite for CacheOp {
    type Db<'a> = CacheAdapter;
    type Input = Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<()> {
        // `Ok` carries no user_uuid, so we refresh company-by-company for every
        // distinct company present in the new data.
        let mut seen: HashSet<String> = HashSet::new();
        for data in &input.data {
            let company_uuid = data.company_uuid.to_string();
            if seen.insert(company_uuid.clone()) {
                txn.tables_db
                    .execute(DELETE_WITH_QUANTITY_QUERY, params![&company_uuid])?;
                txn.tables_db
                    .execute(DELETE_WITHOUT_QUANTITY_QUERY, params![&company_uuid])?;
            }
        }

        for data in &input.data {
            let query = if data.is_it_contain_quantity {
                WRITE_WITH_QUANTITY_QUERY
            } else {
                WRITE_WITHOUT_QUANTITY_QUERY
            };
            txn.tables_db.execute(
                query,
                params![
                    data.row_uuid.to_string(),
                    data.company_uuid.to_string(),
                    &data.name
                ],
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_string_directly() {
        test_query_helper_for_tables_schema(READ_QUERY).unwrap();
        test_query_helper_for_tables_schema(DELETE_WITH_QUANTITY_QUERY).unwrap();
        test_query_helper_for_tables_schema(DELETE_WITHOUT_QUANTITY_QUERY).unwrap();
        test_query_helper_for_tables_schema(WRITE_WITH_QUANTITY_QUERY).unwrap();
        test_query_helper_for_tables_schema(WRITE_WITHOUT_QUANTITY_QUERY).unwrap();
    }
}
