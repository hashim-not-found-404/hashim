use deadpool_postgres::{Config, Pool, Runtime};
use my_core::traits::{self, DBClient, DBTransaction, Database};
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
    type RowId = impls_for_wasm::a1::RowId;
    type HashedPassword = authentication::HashedPassword;
    type Txn<'a> = CockroachTxn<'a>;

    async fn begin_transaction(&mut self) -> Result<Self::Txn<'_>, Self::Error> {
        Ok(CockroachTxn {
            txn: self.client.transaction().await.unwrap(),
        })
    }

    async fn read_sign_in(
        &mut self,
        user_id: &String,
    ) -> Result<Option<(Self::RowId, Self::HashedPassword)>, Self::Error> {
        let query = "SELECT rowid,pass FROM accounting_app.user WHERE id = $1 limit 1;";
        let stmt = self.client.prepare_cached(query).await.unwrap();
        let result = self.client.query_opt(&stmt, &[user_id]).await;

        match result {
            Ok(Some(row)) => {
                let row_id = row.try_get::<_, Uuid>(0);
                let row_id = match row_id {
                    Ok(o) => o,
                    Err(e) => unreachable!("{}", e),
                };

                let hashed_password = row.try_get::<_, String>(1);
                let hashed_password = match hashed_password {
                    Ok(o) => o,
                    Err(e) => unreachable!("{}", e),
                };

                return Ok(Some((row_id.into(), hashed_password.into())));
            }
            Ok(None) => {
                return Ok(None);
            }
            Err(e) => unreachable!("{}", e),
        }
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

    async fn read_sign_up(&mut self, user_id: &String) -> Result<bool, Self::Error> {
        let query = "SELECT EXISTS( SELECT 1 FROM accounting_app.user WHERE id = $1 );";
        let stmt = self.txn.prepare_cached(query).await.unwrap();
        let result = self.txn.query_one(&stmt, &[user_id]).await;

        match result {
            Ok(row) => {
                let value = row.try_get::<_, bool>(0);
                match value {
                    Ok(o) => return Ok(!o),
                    Err(e) => unreachable!("{}", e),
                }
            }
            Err(e) => unreachable!("{}", e),
        }
    }

    async fn write_sign_up(
        &mut self,
        user_id: &String,
        hashed_password: &Self::HashedPassword,
        user_name: &Option<String>,
    ) -> Result<Self::RowId, Self::Error> {
        let query =
            "INSERT INTO accounting_app.user (id, pass, name) VALUES ($1, $2, $3) RETURNING rowid;";
        let stmt = self.txn.prepare_cached(query).await.unwrap();
        let result = self
            .txn
            .query_one(
                &stmt,
                &[&user_id, &hashed_password.into_inner(), &user_name],
            )
            .await;

        match result {
            Ok(row) => {
                let value = row.try_get::<_, Uuid>(0);
                match value {
                    Ok(o) => return Ok(o.into()),
                    Err(e) => unreachable!("{}", e),
                }
            }
            Err(e) => unreachable!("{}", e),
        }
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
