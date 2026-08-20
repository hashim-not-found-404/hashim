use my_core::utility::traits;
use rusqlite::Connection;

pub(crate) fn test_query_helper_for_tables_schema(
    sql_query: &str,
) -> Result<(), traits::DynamicError> {
    let conn = Connection::open_in_memory().unwrap();
    const SCHEMA: &str = include_str!("../../schema/tables.sql");
    conn.execute_batch(SCHEMA).unwrap();
    conn.execute_batch(sql_query)?;
    Ok(())
}

pub(crate) fn test_query_helper_for_transactions_schema(
    sql_query: &str,
) -> Result<(), traits::DynamicError> {
    let conn = Connection::open_in_memory().unwrap();
    const SCHEMA: &str = include_str!("../../schema/transactions.sql");
    conn.execute_batch(SCHEMA).unwrap();
    conn.execute_batch(sql_query)?;
    Ok(())
}
