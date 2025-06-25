use super::DatabaseObject;

/// A view is a virtual table that is based on the result of a SELECT query.
/// It allows users to simplify complex queries, encapsulate data, and restrict
/// access to specific rows or columns of a table without storing the data physically.
pub trait View: DatabaseObject {}
