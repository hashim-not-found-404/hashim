use crate::prelude::*;
use adapters::row_id::m::MyUuidConverter;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::str::FromStr;
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

    async fn read_sign_up(
        &mut self,
        new_uuid: &db_types::UuidType,
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
            .query_one(&stmt, &[&new_uuid.into_inner(), user_id])
            .await
            .log()?;

        let uuid_exists: bool = row.try_get("uuid_exists").log()?;
        let user_id_exists: bool = row.try_get("user_id_exists").log()?;

        Ok((uuid_exists, user_id_exists))
    }

    async fn write_sign_up(
        &mut self,
        new_uuid: &db_types::UuidType,
        user_id: &String,
        hashed_password: &String,
        user_name: &Option<String>,
    ) -> Result<(), DynamicError> {
        let query =
            "INSERT INTO accounting_app.user (rowid, id, pass, name) VALUES ($1, $2, $3, $4)";

        let stmt = self.txn.prepare_cached(query).await.log()?;

        self.txn
            .execute(
                &stmt,
                &[&new_uuid.into_inner(), user_id, &hashed_password, user_name],
            )
            .await
            .log()?;

        Ok(())
    }

    async fn read_create_company(
        &mut self,
        new_uuid: &db_types::UuidType,
    ) -> Result<bool /* is new_uuid exist */, DynamicError> {
        let query = "SELECT EXISTS(SELECT 1 FROM accounting_app.company WHERE rowid = $1)";
        let stmt = self.txn.prepare_cached(query).await.log()?;
        let row = self
            .txn
            .query_one(&stmt, &[&new_uuid.into_inner()])
            .await
            .log()?;

        let exists: bool = row.try_get(0).log()?;
        Ok(exists)
    }

    async fn write_create_company(
        &mut self,
        new_uuid: &db_types::UuidType,
        user_uuid: &db_types::UuidType,
        user_role: &db_types::Role,
        company_name: &String,
        currency: &db_types::Currency,
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
        let row = self
            .txn
            .execute(
                &stmt,
                &[
                    &new_uuid.into_inner(),
                    &company_name,
                    &currency.as_str(),
                    &user_uuid.into_inner(),
                    &user_role.as_str(),
                ],
            )
            .await
            .log()?;
        Ok(())
    }

    async fn read_create_company_branch(
        &mut self,
        new_uuid: &db_types::UuidType,
        user_uuid: &db_types::UuidType,
        company_belong: &db_types::UuidType,
        branch_name: &String,
    ) -> Result<
        (
            Vec<db_types::Role>, /* user roles */
            bool,                /* is new_uuid exist */
            bool,                /* is company_belong exist */
            bool,                /* is branch_name used */
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
                    &company_belong.into_inner(),
                    &user_uuid.into_inner(),
                    &new_uuid.into_inner(),
                    &branch_name,
                ],
            )
            .await
            .log()?;

        let role_strings: Vec<String> = row.try_get(0).log()?;
        let roles = role_strings
            .into_iter()
            .map(|s| db_types::Role::from_str(&s))
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
        new_uuid: &db_types::UuidType,
        company_belong: &db_types::UuidType,
        branch_name: &String,
        location: &db_types::Location,
        currency: &db_types::Currency,
        user_uuid: &db_types::UuidType,
        user_role: &db_types::Role,
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

        let lat = Decimal::from_f64(location.latitude)
            .ok_or(HashimError::InternalServerError)
            .log()?;
        let lng = Decimal::from_f64(location.longitude)
            .ok_or(HashimError::InternalServerError)
            .log()?;

        self.txn
            .execute(
                query,
                &[
                    &new_uuid.into_inner(),
                    &company_belong.into_inner(),
                    &branch_name,
                    &lat,
                    &lng,
                    &currency.as_str(),
                    &user_uuid.into_inner(),
                    &user_role.as_str(),
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
