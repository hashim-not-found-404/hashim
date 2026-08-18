use rusqlite::Connection;

pub(crate) fn test_query_helper(sql_query: &str) {
    let conn = Connection::open_in_memory().unwrap();
    const SCHEMA: &str = include_str!("../../schema/tables.sql");
    conn.execute_batch(SCHEMA).unwrap();
    conn.prepare(sql_query).unwrap();
}
