use crate::prelude::*;
use tokio_postgres::error::SqlState;

pub struct S<'a> {
    pub(crate) txn: deadpool_postgres::Transaction<'a>,
}

impl DBTransaction for S<'_> {
    type RowId = row_id::m::S;
    type HashedPassword = authentication::m::S;

    async fn commit_transaction(self) -> Result<Result<(), domain_errors::AtCommit>, DynamicError> {
        match self.txn.commit().await {
            Ok(_) => return Ok(Ok(())),
            Err(e) => {
                if get_sql_state(&e) == SqlState::T_R_SERIALIZATION_FAILURE {
                    return Ok(Err(domain_errors::AtCommit::DataIsChanged));
                }
                return Err(e.into());
            }
        }
    }

    async fn rollback_transaction(self) -> Result<(), DynamicError> {
        self.txn.rollback().await.log()?;
        Ok(())
    }

    async fn read_sign_up(
        &mut self,
        new_uuid: &Self::RowId,
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
        new_uuid: &Self::RowId,
        user_id: &String,
        hashed_password: &Self::HashedPassword,
        user_name: &Option<String>,
    ) -> Result<(), DynamicError> {
        let query =
            "INSERT INTO accounting_app.user (rowid, id, pass, name) VALUES ($1, $2, $3, $4)";

        let stmt = self.txn.prepare_cached(query).await.log()?;

        self.txn
            .execute(
                &stmt,
                &[
                    &new_uuid.into_inner(),
                    user_id,
                    &hashed_password.into_inner(),
                    user_name,
                ],
            )
            .await
            .log()?;

        Ok(())
    }

    async fn read_create_company(
        &mut self,
        new_uuid: &Self::RowId,
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
        new_uuid: &Self::RowId,
        user_uuid: &Self::RowId,
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
        nonce: &Self::RowId,
        company_belong: &Self::RowId,
        branch_name: &String,
    ) -> Result<
        (
            bool, /* is nonce used */
            bool, /* is company_belong exist */
            bool, /* is branch_name used */
        ),
        DynamicError,
    > {
        todo!()
    }

    async fn write_create_company_branch(
        &mut self,
        new_uuid: &Self::RowId,
        company_belong: &Self::RowId,
        branch_name: &String,
        location: &db_types::Location,
        currency: &db_types::Currency,
        user_uuid: &Self::RowId,
        user_role: &db_types::Role,
    ) -> Result<(), DynamicError> {
        todo!()
    }
}

fn get_sql_state(error: &tokio_postgres::Error) -> SqlState {
    return error.as_db_error().unwrap().code().clone();
}
