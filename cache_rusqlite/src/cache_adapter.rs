use crate::prelude::*;
use rusqlite::{Connection, Result};
// use sqlite_wasm_rs::sahpool_vfs::{OpfsSAHPoolCfg, install as install_opfs_sahpool};

pub struct S {
    db: Connection,
}

impl CacheIO for S {
    async fn new() -> Result<Self, DynamicError> {
        // Step 1: Install the OPFS (Origin Private File System) VFS
        // This must be done BEFORE opening any persistent database connection.
        // OPFS provides the best performance for persistent storage in modern browsers[citation:3].
        // install_opfs_sahpool(&OpfsSAHPoolCfg::default(), true)
        //     .await
        //     .expect("Failed to install OPFS VFS");

        // Step 2: Open a connection using the opfs-sahpool VFS
        // The "opfs-sahpool://" prefix tells SQLite to use the OPFS storage backend[citation:6].
        // The database file will persist across page reloads in the browser.
        let conn = Connection::open("opfs-sahpool://my_persistent_database.db")?;

        // Step 3: Create a table (standard SQLite operations)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT UNIQUE
            )",
            [],
        )?;

        // Step 4: Insert data
        conn.execute(
            "INSERT INTO users (name, email) VALUES (?1, ?2)",
            ["Alice Johnson", "alice@example.com"],
        )?;

        // Step 5: Query data
        let mut stmt = conn.prepare("SELECT id, name, email FROM users")?;
        let user_iter = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        for user in user_iter {
            let (id, name, email) = user?;
            println!("Found user: {} - {} (ID: {})", name, email, id);
        }

        let conn = Connection::open_in_memory()?;
        Ok(Self { db: conn })
    }

    async fn get_all_write_txns(
        &self,
    ) -> Result<Vec<push_data::TxnInput<push_data::WriteOperationInput>>, DynamicError> {
        todo!()
    }

    async fn get_jwt(
        &self,
        user_uuid: &db_types::RowIdType,
    ) -> Result<Option<String>, DynamicError> {
        todo!()
    }

    async fn write_data(&self, data: &Vec<ResourceInfo>) -> Result<(), DynamicError> {
        todo!()
    }

    async fn write_txn<T>(&self, txn: &push_data::TxnInput<T>) -> Result<(), DynamicError> {
        todo!()
    }

    async fn get_txn<T>(
        &self,
        user_uuid: &db_types::RowIdType,
        txn_number: &u64,
    ) -> Result<push_data::TxnInput<T>, DynamicError> {
        todo!()
    }

    async fn delete_txn(
        &self,
        user_uuid: &db_types::RowIdType,
        txn_number: &u64,
    ) -> Result<(), DynamicError> {
        todo!()
    }
}
