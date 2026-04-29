use my_core::traits::{self, DBClient, DBTransaction, Database};
use server_logic::*;
use sqlx::{PgPool, Postgres, Row, Transaction, postgres::PgPoolOptions};
use uuid::Uuid;

pub struct CockroachDB {
    pool: PgPool,
}

impl CockroachDB {
    /// Creates a new connection pool to CockroachDB.
    pub async fn new() -> Self {
        // Build connection URL from components
        let host = "localhost";
        let port = 26257;
        let user = "root";
        let database = "accounting_app";

        // Construct the URL
        let url = format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode=disable",
            user,     // No password for root (CockroachDB default)
            "",       // Empty password
            host,     // localhost
            port,     //
            database  // accounting_app
        );

        let pool = PgPoolOptions::new()
            .max_connections(10) // Configure pool size
            .connect(&url)
            .await
            .unwrap();

        CockroachDB { pool }
    }
}

impl Database for CockroachDB {
    type Error = ();
    type Client = CockroachClient;

    async fn get_client(&self) -> Result<Self::Client, Self::Error> {
        Ok(CockroachClient {
            client: self.pool.clone(),
        })
    }
}

pub struct CockroachClient {
    client: PgPool,
}

impl DBClient for CockroachClient {
    type Error = ();
    type Txn<'a> = CockroachTxn<'a>;

    async fn begin_transaction(&mut self) -> Result<Self::Txn<'_>, Self::Error> {
        let txn = self.client.begin().await.unwrap();
        Ok(CockroachTxn { txn })
    }
}

pub struct CockroachTxn<'a> {
    txn: Transaction<'a, Postgres>,
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
                todo!()
            }
        }
    }

    async fn rollback_transaction(self) -> Result<(), Self::Error> {
        self.txn.rollback().await.unwrap();
        Ok(())
    }

    async fn read_sign_up(&mut self, user_id: &String) -> Result<bool, Self::Error> {
        let query = "SELECT EXISTS( SELECT 1 FROM accounting_app.user WHERE id = $1 );";
        let stmt = sqlx::query(query);
        let result = stmt.bind(&user_id).fetch_one(&mut *self.txn).await;

        let result = sqlx::query!(
            "SELECT EXISTS( SELECT 1 FROM accounting_app.user WHERE id = $1 );",
            user_id
        )
        .fetch_one(&mut *self.txn)
        .await;

        match result {
            Ok(row) => {
                let value = row.try_get::<bool, _>(0);
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
        let stmt = sqlx::query(query);
        let result = stmt
            .bind(&user_id)
            .bind(&hashed_password.into_inner())
            .bind(&user_name)
            .fetch_one(&mut *self.txn)
            .await;

        match result {
            Ok(row) => {
                let value = row.try_get::<Uuid, _>(0);
                match value {
                    Ok(o) => return Ok(Self::RowId::try_from(o).unwrap()),
                    Err(e) => unreachable!("{}", e),
                }
            }
            Err(e) => unreachable!("{}", e),
        }
    }
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
