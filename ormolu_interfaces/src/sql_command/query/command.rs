pub enum Command {
    // -- Data Manipulation
    Insert {},
    Update {},
    Delete,
    // LOCK,
    // CALL,
    // EXPLAIN PLAN,

    // -- Data Query
    Select { fields: &'static [&'static str] },
    // -- Data Define
    // Create {},
    // DROP,
    // ALTER,
    // TRUNCATE,
    // COMMENT,
    // RENAME,
}
