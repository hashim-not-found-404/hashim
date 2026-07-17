use my_core::{
    accounting_domain::cases::{self, utility::types},
    server::use_cases::utility::server_traits::{DBTransaction, domain_errors},
    utility::{traits::DynamicError, utils::LogError},
};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use std::str::FromStr;
use tokio_postgres::error::SqlState;

use crate::utils::MyUuidConverter;

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

    async fn read_sign_up(
        &mut self,
        new_uuid: &types::UuidType,
        user_id: &String,
    ) -> Result<
        (
            bool, /* is new_uuid exist */
            bool, /* is user_id exist */
        ),
        DynamicError,
    > {
        let query = "
             SELECT
                 EXISTS(SELECT 1 FROM accounting_app.user WHERE rowid = $1) AS uuid_exists,
                 EXISTS(SELECT 1 FROM accounting_app.user WHERE id = $2) AS user_id_exists
         ";

        let stmt = self.txn.prepare_cached(query).await.log()?;
        let row = self
            .txn
            .query_one(&stmt, &[&new_uuid.to_externel_uuid(), user_id])
            .await
            .log()?;

        let uuid_exists: bool = row.try_get("uuid_exists").log()?;
        let user_id_exists: bool = row.try_get("user_id_exists").log()?;

        Ok((uuid_exists, user_id_exists))
    }

    async fn write_sign_up(&mut self, data: &cases::sign_up::Ok) -> Result<(), DynamicError> {
        let query =
            "INSERT INTO accounting_app.user (rowid, id, pass, name) VALUES ($1, $2, $3, $4)";

        let stmt = self.txn.prepare_cached(query).await.log()?;

        self.txn
            .execute(
                &stmt,
                &[
                    &data.new_uuid.to_externel_uuid(),
                    &data.user_id,
                    &data.hashed_password,
                    &data.user_name,
                ],
            )
            .await
            .log()?;

        Ok(())
    }

    async fn read_create_company(
        &mut self,
        new_uuid: &types::UuidType,
    ) -> Result<bool /* is new_uuid exist */, DynamicError> {
        let query = "SELECT EXISTS(SELECT 1 FROM accounting_app.company WHERE rowid = $1)";
        let stmt = self.txn.prepare_cached(query).await.log()?;
        let row = self
            .txn
            .query_one(&stmt, &[&new_uuid.to_externel_uuid()])
            .await
            .log()?;

        let exists: bool = row.try_get(0).log()?;
        Ok(exists)
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
            .execute(
                &stmt,
                &[
                    &data.new_uuid.to_externel_uuid(),
                    &data.company_name,
                    &data.currency.as_str(),
                    &data.user_uuid.to_externel_uuid(),
                    &data.role.as_str(),
                ],
            )
            .await
            .log()?;
        Ok(())
    }

    async fn read_create_company_branch(
        &mut self,
        new_uuid: &types::UuidType,
        user_uuid: &types::UuidType,
        company_belong: &types::UuidType,
        branch_name: &String,
    ) -> Result<
        (
            Vec<types::Role>, /* user roles */
            bool,             /* is new_uuid exist */
            bool,             /* is company_belong exist */
            bool,             /* is branch_name used */
        ),
        DynamicError,
    > {
        let query = "
            WITH user_roles AS (
                SELECT array_agg(role) as roles
                FROM accounting_app.access_control_for_company
                WHERE data_group = $1 AND user_ = $2
            ),
            checks AS (
                SELECT
                    EXISTS(SELECT 1 FROM accounting_app.company_branch WHERE rowid = $3) as new_uuid_exists,
                    EXISTS(SELECT 1 FROM accounting_app.company WHERE rowid = $1) as company_exists,
                    EXISTS(SELECT 1 FROM accounting_app.company_branch
                          WHERE company_belong = $1 AND name = $4) as branch_name_used
            )
            SELECT
                COALESCE((SELECT roles FROM user_roles), '{}'::text[]) as roles,
                (SELECT new_uuid_exists FROM checks) as new_uuid_exists,
                (SELECT company_exists FROM checks) as company_exists,
                (SELECT branch_name_used FROM checks) as branch_name_used
        ";

        let row = self
            .txn
            .query_one(
                query,
                &[
                    &company_belong.to_externel_uuid(),
                    &user_uuid.to_externel_uuid(),
                    &new_uuid.to_externel_uuid(),
                    &branch_name,
                ],
            )
            .await
            .log()?;

        let role_strings: Vec<String> = row.try_get(0).log()?;
        let roles = role_strings
            .into_iter()
            .map(|s| types::Role::from_str(&s))
            .collect::<Result<Vec<_>, _>>()
            .log()?;
        let is_new_uuid_exist: bool = row.try_get(1).log()?;
        let is_company_belong_exist: bool = row.try_get(2).log()?;
        let is_branch_name_used: bool = row.try_get(3).log()?;

        Ok((
            roles,
            is_new_uuid_exist,
            is_company_belong_exist,
            is_branch_name_used,
        ))
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
            .execute(
                query,
                &[
                    &data.new_uuid.to_externel_uuid(),
                    &data.company_belong.to_externel_uuid(),
                    &data.branch_name,
                    &lat,
                    &lng,
                    &data.currency.as_str(),
                    &data.user_uuid.to_externel_uuid(),
                    &data.role.as_str(),
                ],
            )
            .await
            .log()?;

        Ok(())
    }
}

fn get_sql_state(error: &tokio_postgres::Error) -> SqlState {
    error.as_db_error().unwrap().code().clone()
}
