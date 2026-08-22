use crate::utility::db_client;
use deadpool_postgres::Config;
use deadpool_postgres::Pool;
use deadpool_postgres::Runtime;
use my_core::server::server_methods::Database;
use my_core::utility::traits::DynamicError;
use my_core::utility::utils::LogError;
use tokio_postgres::NoTls;

pub struct S {
    pool: Pool,
}

impl Database for S {
    type Client = db_client::S;

    async fn new() -> Self {
        let mut cfg = Config::new();

        cfg.host = Some("localhost".to_string());
        cfg.port = Some(26257);
        cfg.user = Some("root".to_string());
        cfg.dbname = Some("accounting_app".to_string());

        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

        S {
            pool,
        }
    }

    async fn get_client(&self) -> Result<Self::Client, DynamicError> {
        Ok(db_client::S {
            client: self.pool.get().await.log()?,
        })
    }
}
