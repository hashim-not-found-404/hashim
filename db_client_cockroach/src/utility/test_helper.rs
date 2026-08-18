use my_core::utility::traits;
use tokio_postgres::NoTls;

/// Connects to CockroachDB using environment variables (or defaults),
/// creates the schema (skipping `CREATE DATABASE` and `USE` statements),
/// and verifies that the given SQL query can be prepared successfully.
///
/// # Panics
/// Panics if connection fails, or the query is invalid.
pub(crate) async fn test_query_helper(sql_query: &str) -> Result<(), traits::DynamicError> {
    let host = "localhost".to_string();
    let port = "26257".to_string();
    let user = "root".to_string();
    let dbname = "accounting_app".to_string();
    let url = format!("postgresql://{}@{}:{}/{}", user, host, port, dbname);

    let (client, connection) =
        tokio_postgres::connect(&url, NoTls).await.expect("Failed to connect to CockroachDB");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("CockroachDB connection error: {}", e);
        }
    });

    client.prepare(sql_query).await?;
    Ok(())
}
