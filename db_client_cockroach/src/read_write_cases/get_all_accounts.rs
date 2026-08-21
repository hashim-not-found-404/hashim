use crate::utility::db_client;
use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types::DatabaseRead;
use my_core::accounting_domain::utility::types::{self};
use my_core::utility::traits;
use my_core::utility::utils::LogError;
use uuid::Uuid;

const QUERY1: &str = "
    SELECT
        rowid::text,
        is_debit,
        is_permanent_account,
        name as account_name,
        notes,
        unit_of_measurement_of_quantity
    FROM accounting_app.account
    WHERE belong_to_company = $1
    ORDER BY name
";

pub struct S;

impl cases::get_all_accounts::DatabaseRead for S {}

impl DatabaseRead for S {
    type Db<'a> = db_client::S;
    type Error = traits::DynamicError;
    type Input = cases::get_all_accounts::ReadInput;
    type Output = cases::get_all_accounts::ReadOutput;

    async fn read(
        db: &mut Self::Db<'_>,
        read_input: &Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        let rows =
            db.client.query(QUERY1, &[&read_input.company_uuid.to_externel_uuid()]).await.log()?;

        let mut data = Vec::with_capacity(rows.len());
        for row in rows {
            let row_uuid_str: String = row.try_get(0).log()?;
            let row_uuid_parsed = Uuid::parse_str(&row_uuid_str).log()?;
            let row_uuid = types::UuidType(row_uuid_parsed.into_bytes());

            let is_debit: bool = row.try_get(1).log()?;
            let is_permanent_account: bool = row.try_get(2).log()?;
            let account_name: String = row.try_get(3).log()?;
            let notes: Option<String> = row.try_get(4).log()?;
            let unit_of_measurement_of_quantity: String = row.try_get(5).log()?;

            data.push(cases::get_all_accounts::Data {
                row_uuid,
                is_debit,
                is_permanent_account,
                account_name,
                notes,
                unit_of_measurement_of_quantity,
            });
        }

        Ok(cases::get_all_accounts::ReadOutput {
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
        test_query_helper(QUERY1).await.unwrap();
    }
}
