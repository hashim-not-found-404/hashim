use my_core::utility::traits;
use rusqlite::Connection;

pub(crate) fn test_query_helper(sql_query: &str) -> Result<(), traits::DynamicError> {
    let conn = Connection::open_in_memory().unwrap();
    const SCHEMA: &str = include_str!("../../schema/tables.sql");
    conn.execute_batch(SCHEMA).unwrap();
    conn.prepare(sql_query)?;
    Ok(())
}
