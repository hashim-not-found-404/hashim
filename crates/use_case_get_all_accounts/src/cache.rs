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
use rusqlite::params;

const READ_QUERY: &str = "
    SELECT rowid, is_debit, is_permanent_account, name,
           unit_of_measurement_of_quantity
    FROM account
    WHERE belong_to_company = ?1
    ORDER BY name
";

const DELETE_QUERY: &str = "DELETE FROM account WHERE belong_to_company = ?1";

const WRITE_QUERY: &str = "
    INSERT OR REPLACE INTO account (
        rowid, is_debit, is_permanent_account, name,
        unit_of_measurement_of_quantity, belong_to_company
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
";

pub struct CacheOp;

impl DatabaseRead for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = ReadInput;
    type Output = ReadOutput;

    async fn read(db: &mut Self::Db<'_>, input: &Self::Input) -> Result<Self::Output> {
        let mut stmt = db.tables_db.prepare(READ_QUERY)?;
        let rows = stmt.query_map(params![input.company_uuid.to_string()], |row| {
            let row_uuid_str: String = row.get(0)?;
            let is_debit: bool = row.get(1)?;
            let is_permanent_account: bool = row.get(2)?;
            let account_name: String = row.get(3)?;
            let unit_of_measurement_of_quantity: String = row.get(5)?;
            Ok((
                row_uuid_str,
                is_debit,
                is_permanent_account,
                account_name,
                unit_of_measurement_of_quantity,
            ))
        })?;

        let mut data = Vec::new();
        for row in rows {
            let (
                row_uuid_str,
                is_debit,
                is_permanent_account,
                account_name,
                unit_of_measurement_of_quantity,
            ) = row?;
            data.push(Data {
                row_uuid: row_uuid_str.to_uuid()?.into(),
                is_debit,
                is_permanent_account,
                account_name,
                unit_of_measurement_of_quantity,
            });
        }

        Ok(ReadOutput { data })
    }
}

impl DatabaseWrite for CacheOp {
    type Db<'a> = cache_adapter::S;
    type Input = Ok;

    async fn write(txn: &mut Self::Db<'_>, input: &Self::Input) -> Result<()> {
        let company_uuid = input.company_uuid.to_string();

        txn.tables_db
            .execute(DELETE_QUERY, params![&company_uuid])?;

        for data in &input.data {
            txn.tables_db.execute(
                WRITE_QUERY,
                params![
                    data.row_uuid.to_string(),
                    data.is_debit,
                    data.is_permanent_account,
                    &data.account_name,
                    &data.unit_of_measurement_of_quantity,
                    &company_uuid,
                ],
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
        test_query_helper_for_tables_schema(DELETE_QUERY).unwrap();
        test_query_helper_for_tables_schema(WRITE_QUERY).unwrap();
    }
}
