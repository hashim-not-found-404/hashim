use crate::utility::utils::MyUuidConverter;
use my_core::accounting_domain::cases;
use my_core::accounting_domain::utility::types;
use my_core::server::utility::server_traits::DBTransaction;
use my_core::server::utility::server_traits::domain_errors;
use my_core::utility::traits::DynamicError;
use my_core::utility::utils::LogError;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use tokio_postgres::error::SqlState;

pub struct S<'a> {
    pub(crate) txn: deadpool_postgres::Transaction<'a>,
}

impl DBTransaction for S<'_> {
    async fn commit_transaction(self) -> Result<Result<(), domain_errors::AtCommit>, DynamicError> {
        match self.txn.commit().await {
            Ok(_) => Ok(Ok(())),
            Err(e) => {
                if get_sql_state(&e) == SqlState::T_R_SERIALIZATION_FAILURE {
                    return Ok(Err(domain_errors::AtCommit::DataIsChanged));
                }
                Err(e.into())
            }
        }
    }

    async fn rollback_transaction(self) -> Result<(), DynamicError> {
        self.txn.rollback().await.log()?;
        Ok(())
    }

    async fn write_sign_up(&mut self, data: &cases::sign_up::Ok) -> Result<(), DynamicError> {
        let query =
            "INSERT INTO accounting_app.user (rowid, id, pass, name) VALUES ($1, $2, $3, $4)";

        let stmt = self.txn.prepare_cached(query).await.log()?;

        self.txn
            .execute(&stmt, &[
                &data.new_uuid.to_externel_uuid(),
                &data.user_id,
                &data.hashed_password,
                &data.user_name,
            ])
            .await
            .log()?;

        Ok(())
    }

    async fn write_create_company(
        &mut self,
        data: &cases::create_company::Ok,
    ) -> Result<(), DynamicError> {
        let query = "
            WITH company_insert AS (
                INSERT INTO accounting_app.company (rowid, name, currency)
                VALUES ($1, $2, $3)
                RETURNING 1
            )
            INSERT INTO accounting_app.access_control_for_company (rowid, data_group, user_, role)
            VALUES ($1, $1, $4, $5)
            ;";

        let stmt = self.txn.prepare_cached(query).await.log()?;
        let _row = self
            .txn
            .execute(&stmt, &[
                &data.new_uuid.to_externel_uuid(),
                &data.company_name,
                &data.currency.as_str(),
                &data.user_uuid.to_externel_uuid(),
                &data.role.as_str(),
            ])
            .await
            .log()?;
        Ok(())
    }

    async fn write_create_company_branch(
        &mut self,
        data: &cases::create_company_branch::Ok,
    ) -> Result<(), DynamicError> {
        let query = "
            WITH inserted_branch AS (
                INSERT INTO accounting_app.company_branch (
                    rowid, company_belong, name,
                    location_latitude, location_longitude, currency
                ) VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING rowid
            )
            INSERT INTO accounting_app.access_control_for_company_branch (
                rowid, data_group, user_, role
            )
            SELECT rowid, rowid, $7, $8 FROM inserted_branch
        ";

        let lat = Decimal::from_f64(data.location.latitude)
            .ok_or(types::HashimError::InternalServerError)
            .log()?;
        let lng = Decimal::from_f64(data.location.longitude)
            .ok_or(types::HashimError::InternalServerError)
            .log()?;

        self.txn
            .execute(query, &[
                &data.new_uuid.to_externel_uuid(),
                &data.company_belong.to_externel_uuid(),
                &data.branch_name,
                &lat,
                &lng,
                &data.currency.as_str(),
                &data.user_uuid.to_externel_uuid(),
                &data.role.as_str(),
            ])
            .await
            .log()?;

        Ok(())
    }

    async fn write_create_account(
        &mut self,
        input: &cases::create_account::Ok,
    ) -> Result<(), DynamicError> {
        let query = "
            INSERT INTO accounting_app.account (
                rowid,
                is_debit,
                is_permanent_account,
                name,
                notes,
                belong_to_company,
                unit_of_measurement_of_quantity
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        ";

        let stmt = self.txn.prepare_cached(query).await.log()?;
        self.txn
            .execute(&stmt, &[
                &input.new_uuid.to_externel_uuid(),
                &input.is_debit,
                &input.is_permanent_account,
                &input.account_name,
                &input.notes,
                &input.belong_to_company.to_externel_uuid(),
                &input.unit_of_measurement_of_quantity,
            ])
            .await
            .log()?;

        Ok(())
    }
}

fn get_sql_state(error: &tokio_postgres::Error) -> SqlState {
    error.as_db_error().unwrap().code().clone()
}
