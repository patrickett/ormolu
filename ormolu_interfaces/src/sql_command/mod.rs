// https://www.geeksforgeeks.org/sql-ddl-dql-dml-dcl-tcl-commands/
pub mod query;
// use query::*;

pub enum Command {
    // -- Data Manipulation
    Insert {},
    Update {},
    Delete,
    // LOCK,
    // CALL,
    // EXPLAIN PLAN,

    // -- Data Query
    Select { columns: &'static [&'static str] },
    // -- Data Define
    // Create {},
    // DROP,
    // ALTER,
    // TRUNCATE,
    // COMMENT,
    // RENAME,
}
