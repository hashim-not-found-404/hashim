use crate::db_client::DbClientAdapter;
use anyhow::Result;
use deadpool_postgres::Config;
use deadpool_postgres::Pool;
use deadpool_postgres::Runtime;
use kernel::server::Database;
use tokio_postgres::NoTls;
use utility::types::LogError;

pub struct DbAdapter {
    pool: Pool,
}

impl Database for DbAdapter {
    type Client = DbClientAdapter;

    async fn new() -> Result<Self> {
        let mut cfg = Config::new();

        cfg.host = Some("localhost".to_string());
        cfg.port = Some(26257);
        cfg.user = Some("root".to_string());
        cfg.dbname = Some("accounting_app".to_string());

        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

        Ok(DbAdapter { pool })
    }

    async fn get_client(&self) -> Result<Self::Client> {
        Ok(DbClientAdapter {
            client: self.pool.get().await.log()?,
        })
    }
}
