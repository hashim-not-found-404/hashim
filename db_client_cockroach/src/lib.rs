use deadpool_postgres::{Config, Pool, Runtime};
use my_core::{
    db_types,
    request_response::*,
    traits::{self, DBClient, DBTransaction, Database},
};
use serde_json;
use server_logic::*;
use tokio_postgres::{NoTls, Row, error::SqlState, types::ToSql};
use uuid::Uuid;

pub struct CockroachDB {
    pool: Pool,
}

impl Default for CockroachDB {
    /// Creates a new connection pool to CockroachDB.
    fn default() -> Self {
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
}

pub struct CockroachClient {
    client: deadpool_postgres::Object,
}

impl Database for CockroachDB {
    type Error = ();
    type Client = CockroachClient;

    async fn get_client(&self) -> Result<Self::Client, Self::Error> {
        Ok(CockroachClient {
            client: self.pool.get().await.unwrap(),
        })
    }
}

impl DBClient for CockroachClient {
    type Error = ();
    type Txn<'a> = CockroachTxn<'a>;

    async fn begin_transaction(&mut self) -> Result<Self::Txn<'_>, Self::Error> {
        Ok(CockroachTxn {
            txn: self.client.transaction().await.unwrap(),
        })
    }
}

pub struct CockroachTxn<'a> {
    txn: deadpool_postgres::Transaction<'a>,
}

impl DBTransaction for CockroachTxn<'_> {
    type Error = ();
    type RowId = impls_for_wasm::a1::RowId;
    type HashedPassword = authentication::HashedPassword;

    async fn commit_transaction(
        self,
    ) -> Result<Result<(), traits::domain_errors::AtCommit>, Self::Error> {
        match self.txn.commit().await {
            Ok(_) => return Ok(Ok(())),
            Err(e) => {
                if get_sql_state(e) == SqlState::T_R_SERIALIZATION_FAILURE {
                    return Ok(Err(traits::domain_errors::AtCommit::DataIsChanged));
                }
                unreachable!()
            }
        }
    }

    async fn rollback_transaction(self) -> Result<(), Self::Error> {
        self.txn.rollback().await.unwrap();
        Ok(())
    }

    async fn insert_user(
        &mut self,
        row_id: &Self::RowId,
        name: &Option<String>,
        user_id: &String,
        hashed_password: &Self::HashedPassword,
    ) -> Result<Result<(), traits::domain_errors::AtInsertUserId>, Self::Error> {
        let query = "insert into accounting_app.user (rowid,name,id,pass) values ($1,$2,$3,$4);";

        let stmt = self.txn.prepare_cached(query).await.unwrap();

        let r = self
            .txn
            .execute(
                &stmt,
                &[
                    &row_id.into_inner(),
                    name,
                    user_id,
                    &hashed_password.into_inner(),
                ],
            )
            .await;

        match r {
            Ok(_) => return Ok(Ok(())),
            Err(e) => {
                if get_sql_state(e) == SqlState::UNIQUE_VIOLATION {
                    return Ok(Err(traits::domain_errors::AtInsertUserId::DuplicatedUserId));
                }
                unreachable!()
            }
        }
    }

    async fn does_he_have_access_to_here(
        &mut self,
        accepted_roles: &[custom_types::Role],
        company_or_branch: &db_types::DataGroup<Self::RowId>,
        user_id: &Self::RowId,
    ) -> Result<bool, Self::Error> {
        todo!()
    }
    async fn insert_company(
        &mut self,
        row_id: &Self::RowId,
        name: &String,
        currency: &custom_types::Currency,
    ) -> Result<(), Self::Error> {
        // TODO
        let query = "insert into accounting_app.company (rowid,name,currency) values ($1,$2,$3);";

        let stmt = self.txn.prepare_cached(query).await.unwrap();

        let r = self
            .txn
            .execute(&stmt, &[&row_id.into_inner(), name, &currency.as_str()])
            .await;

        match r {
            Ok(_) => return Ok(()),
            Err(e) => {
                unreachable!()
            }
        }
    }
    async fn insert_company_branch(
        &mut self,
        row_id: &Self::RowId,
        company_belong: &Self::RowId,
        name: &String,
        location: &custom_types::Location,
        currency: &custom_types::Currency,
    ) -> Result<(), Self::Error> {
        todo!()
    }
    async fn insert_role(
        &mut self,
        row_id: &Self::RowId,
        role: &custom_types::Role,
        data_group: &db_types::DataGroup<Self::RowId>,
        user_id: &Self::RowId,
    ) -> Result<(), Self::Error> {
        todo!()
    }
    async fn insert_transaction_if_new(
        &mut self,
        transaction_number: u64,
    ) -> Result<bool, Self::Error> {
        let txn_num = transaction_number as i64;
        let a = self
            .query_opt(
                "INSERT INTO accounting_app.transaction_number (rowid, time)
                        VALUES ($1, NOW())
                        ON CONFLICT (rowid) DO NOTHING
                        RETURNING true",
                &[&txn_num],
            )
            .await;

        Ok(a.is_some())
    }
    async fn select_user_rowid_and_password_hash(
        &mut self,
        user_id: &String,
    ) -> Result<Option<(Self::RowId, Self::HashedPassword)>, Self::Error> {
        let query = "select rowid , pass from accounting_app.user where id = $1;";
        let stmt = self.txn.prepare_cached(query).await.unwrap();

        let r = self.txn.query_opt(&stmt, &[user_id]).await.unwrap();

        match r {
            Some(row) => {
                let a: Uuid = row.get(0);
                let rowid = Self::RowId::from(a);
                let a: String = row.get(1);
                let hashed_password = Self::HashedPassword::from(a);
                return Ok(Some((rowid, hashed_password)));
            }
            None => return Ok(None),
        }
    }

    async fn select_all_companies_and_branches_for_the_user(
        &mut self,
        user_id: &Self::RowId,
    ) -> Result<Option<Vec<custom_types::Company>>, Self::Error> {
        let rows = self
            .txn
            .query(
                r#"
                 SELECT
                     c.rowid,
                     c.name,
                     c.currency,
                     acc.role as user_role,
                     jsonb_agg(
                         DISTINCT jsonb_build_object(
                             'name', cb.name,
                             'location', jsonb_build_object(
                                 'latitude', cb.location_latitude,
                                 'longitude', cb.location_longitude
                             ),
                             'currency', cb.currency,
                             'role', acb.role
                         )
                     ) FILTER (WHERE cb.rowid IS NOT NULL) as branches
                 FROM accounting_app.company c
                 INNER JOIN accounting_app.access_control_for_company acc
                     ON c.rowid = acc.data_group AND acc.user_ = $1
                 LEFT JOIN accounting_app.company_branch cb
                     ON c.rowid = cb.company_belong
                 LEFT JOIN accounting_app.access_control_for_company_branch acb
                     ON cb.rowid = acb.data_group AND acb.user_ = $1
                 GROUP BY c.rowid, c.name, c.currency, acc.role
                 "#,
                &[&user_id.into_inner()],
            )
            .await
            .unwrap();

        // TODO create the page in dioxus first
        dbg!(&rows, "// TODO create the page in dioxus first");

        let mut companies = Vec::with_capacity(rows.len());
        for row in rows {
            let name: String = row.get("name");
            // let currency: serde_json::Value = row.get("currency");
            // let currency: custom_types::Currency = serde_json::from_value(currency).unwrap();
            // let role: serde_json::Value = row.get("user_role");
            // let role: custom_types::Role = serde_json::from_value(role).unwrap();
            // let branches: serde_json::Value = row.get("branches");
            // let branches: Vec<custom_types::Branch> = serde_json::from_value(branches).unwrap();

            // companies.push(custom_types::Company {
            //     name,
            //     currency,
            //     role,
            //     branches,
            // });
        }

        Ok(Some(companies))
    }
}

impl CockroachTxn<'_> {
    async fn execute(&self, query: &str, params: &[&(dyn ToSql + Sync)]) {
        let stmt = self.txn.prepare_cached(query).await.unwrap();
        self.txn.execute(&stmt, params).await.unwrap();
    }

    async fn query_opt(&self, query: &str, params: &[&(dyn ToSql + Sync)]) -> Option<Row> {
        let stmt = self.txn.prepare_cached(query).await.unwrap();
        return self.txn.query_opt(&stmt, params).await.unwrap();
    }
}

fn get_sql_state(error: tokio_postgres::Error) -> SqlState {
    return error.as_db_error().unwrap().code().clone();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_user() {
        test_suite::test_insert_user::<
            CockroachDB,
            impls_for_wasm::a1::RowId,
            server_logic::authentication::HashedPassword,
        >()
        .await;
    }
}
