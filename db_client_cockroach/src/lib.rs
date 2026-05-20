use adapters::prelude::*;
use deadpool_postgres::{Config, Pool, Runtime};
use my_core::prelude::*;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, str::FromStr};
use tokio_postgres::types::Timestamp;
use tokio_postgres::{NoTls, error::SqlState};
use uuid::Uuid;

pub struct CockroachDB {
    pool: Pool,
}

impl Database for CockroachDB {
    type Client = CockroachClient;

    async fn new() -> Self {
        let mut cfg = Config::new();

        // Connection settings
        cfg.host = Some("localhost".to_string());
        cfg.port = Some(26257);
        cfg.user = Some("root".to_string());
        cfg.dbname = Some("accounting_app".to_string());

        let pool = cfg
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .expect("Failed to create database pool");

        CockroachDB { pool: pool }
    }

    async fn get_client(&self) -> Result<Self::Client, DynamicError> {
        Ok(CockroachClient {
            client: self.pool.get().await?,
        })
    }
}

pub struct CockroachClient {
    client: deadpool_postgres::Object,
}

impl DBClient for CockroachClient {
    type RowId = row_id::m::S;
    type HashedPassword = authentication::m::S;
    type Txn<'a> = CockroachTxn<'a>;

    async fn begin_transaction(&mut self) -> Result<Self::Txn<'_>, DynamicError> {
        Ok(CockroachTxn {
            txn: self.client.transaction().await?,
        })
    }

    async fn read_sign_in(
        &mut self,
        user_id: &String,
    ) -> Result<Option<(Self::RowId, Self::HashedPassword)>, DynamicError> {
        todo!();
        // let query = "SELECT rowid,pass FROM accounting_app.user WHERE id = $1 LIMIT 1;";
        // let stmt = self.client.prepare_cached(query).await?;
        // let row = self.client.query_opt(&stmt, &[user_id]).await?;

        // match row {
        //     Some(row) => {
        //         let row_id = row.try_get::<_, Uuid>(0)?;
        //         let hashed_password = row.try_get::<_, String>(1)?;
        //         return Ok(Some((row_id.into(), hashed_password.into())));
        //     }
        //     None => {
        //         return Ok(None);
        //     }
        // }
    }

    async fn read_roles_for_user(
        &mut self,
        user_uuid: &HashSet<Self::RowId>,
    ) -> Result<server_methods::AllRoles<Self::RowId>, DynamicError> {
        todo!();
        // let query = r#"
        //     SELECT
        //         'company' as type,
        //         data_group,
        //         role
        //     FROM accounting_app.access_control_for_company
        //     WHERE user_ = $1

        //     UNION ALL

        //     SELECT
        //         'branch' as type,
        //         data_group,
        //         role
        //     FROM accounting_app.access_control_for_company_branch
        //     WHERE user_ = $1
        // "#;

        // let stmt = self.client.prepare_cached(query).await?;
        // let rows = self.client.query(&stmt, &[&user_uuid.into_inner()]).await?;

        // let mut result = server_methods::AllRoles::<Self::RowId> {
        //     companies: HashMap::new(),
        //     branches: HashMap::new(),
        // };

        // for row in rows {
        //     let entity_type: String = row.try_get("type")?;
        //     let data_group: Uuid = row.try_get("data_group")?;
        //     let role_str: &str = row.try_get("role")?;

        //     let role = db_types::Role::from_str(role_str)?;

        //     match entity_type.as_str() {
        //         "company" => {
        //             result
        //                 .companies
        //                 .entry(data_group.into())
        //                 .or_insert_with(Vec::new)
        //                 .push(role);
        //         }
        //         "branch" => {
        //             result
        //                 .branches
        //                 .entry(data_group.into())
        //                 .or_insert_with(Vec::new)
        //                 .push(role);
        //         }
        //         _ => {}
        //     }
        // }

        // Ok(result)
    }

    async fn write_nonce_if_not_used(
        &mut self,
        nonce: &Self::RowId,
    ) -> Result<bool /* is nonce used */, DynamicError> {
        todo!()
    }
}

pub struct CockroachTxn<'a> {
    txn: deadpool_postgres::Transaction<'a>,
}

impl DBTransaction for CockroachTxn<'_> {
    type RowId = row_id::m::S;
    type HashedPassword = authentication::m::S;

    async fn commit_transaction(self) -> Result<Result<(), domain_errors::AtCommit>, DynamicError> {
        match self.txn.commit().await {
            Ok(_) => return Ok(Ok(())),
            Err(e) => {
                if get_sql_state(e) == SqlState::T_R_SERIALIZATION_FAILURE {
                    return Ok(Err(domain_errors::AtCommit::DataIsChanged));
                }
                unreachable!()
            }
        }
    }

    async fn rollback_transaction(self) -> Result<(), DynamicError> {
        self.txn.rollback().await?;
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
        // Query for both existence checks in one go
        let query = "
             SELECT
                 EXISTS(SELECT 1 FROM accounting_app.user WHERE rowid = $1) AS uuid_exists,
                 EXISTS(SELECT 1 FROM accounting_app.user WHERE id = $2) AS user_id_exists
         ";

        let stmt = self.txn.prepare_cached(query).await?;
        let row = self
            .txn
            .query_one(&stmt, &[&new_uuid.into_inner(), user_id])
            .await?;

        let uuid_exists: bool = row.try_get("uuid_exists")?;
        let user_id_exists: bool = row.try_get("user_id_exists")?;

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

        let stmt = self.txn.prepare_cached(query).await?;

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
            .await?;

        Ok(())
    }

    async fn read_create_company(&mut self, nonce: &Self::RowId) -> Result<bool, DynamicError> {
        todo!();
        // let query =
        //     "SELECT EXISTS(SELECT 1 FROM accounting_app.transaction_number WHERE rowid = $1)";
        // let stmt = self.txn.prepare_cached(query).await?;
        // let row = self.txn.query_one(&stmt, &[&nonce.into_inner()]).await?;

        // let exists: bool = row.try_get(0)?;
        // Ok(exists)
    }

    async fn write_create_company(
        &mut self,
        resources: &mut Vec<ResourceInfo>,
        new_uuid: &Self::RowId,
        user_uuid: &Self::RowId,
        user_role: &db_types::Role,
        company_name: &String,
        currency: &db_types::Currency,
    ) -> Result<(), DynamicError> {
        todo!();
        // let query = "
        //     WITH

        //     nonce_insert AS (
        //         INSERT INTO accounting_app.transaction_number (rowid) VALUES ($1)
        //         RETURNING 1
        //     ),

        //     company_insert AS (
        //         INSERT INTO accounting_app.company (name, currency) VALUES ($2, $3)
        //         RETURNING rowid, updated_at
        //     )

        //     INSERT INTO accounting_app.access_control_for_company (data_group, user_, role)
        //     SELECT company_insert.rowid, $4, $5 FROM company_insert

        //     RETURNING
        //         (SELECT rowid FROM company_insert) AS company_rowid,
        //         (SELECT updated_at FROM company_insert) AS company_updated_at,
        //         rowid AS access_control_rowid,
        //         updated_at AS access_control_updated_at
        //     ;
        //     ";

        // let stmt = self.txn.prepare_cached(query).await?;
        // let row = self
        //     .txn
        //     .query_one(
        //         &stmt,
        //         &[
        //             &nonce.into_inner(),
        //             &company_name,
        //             &currency.as_str(),
        //             &user_uuid.into_inner(),
        //             &user_role.as_str(),
        //         ],
        //     )
        //     .await?;

        // let company_rowid: Uuid = row.try_get(0)?;
        // let company_updated_at: SystemTime = row.try_get(1)?;
        // let access_control_for_company_rowid: Uuid = row.try_get(2)?;
        // let access_control_for_company_updated_at: SystemTime = row.try_get(3)?;

        // resources.push(ResourceInfo {
        //     version: company_updated_at.duration_since(UNIX_EPOCH)?.as_micros() as u64,
        //     uuid: company_rowid.to_string(),
        //     resource: server_methods::Resource::CompanyName(company_name.clone()),
        // });
        // resources.push(ResourceInfo {
        //     version: company_updated_at.duration_since(UNIX_EPOCH)?.as_micros() as u64,
        //     uuid: company_rowid.to_string(),
        //     resource: server_methods::Resource::CompanyCurrency(currency.clone()),
        // });
        // resources.push(ResourceInfo {
        //     version: access_control_for_company_updated_at
        //         .duration_since(UNIX_EPOCH)?
        //         .as_micros() as u64,
        //     uuid: access_control_for_company_rowid.to_string(),
        //     resource: server_methods::Resource::RoleAtCompany(user_role.clone()),
        // });
        // resources.push(ResourceInfo {
        //     version: access_control_for_company_updated_at
        //         .duration_since(UNIX_EPOCH)?
        //         .as_micros() as u64,
        //     uuid: access_control_for_company_rowid.to_string(),
        //     resource: server_methods::Resource::UserThatHaveRole(
        //         user_uuid.into_inner().to_string(),
        //     ),
        // });

        // Ok(())
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
        resources: &mut Vec<ResourceInfo>,
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

fn get_sql_state(error: tokio_postgres::Error) -> SqlState {
    return error.as_db_error().unwrap().code().clone();
}
