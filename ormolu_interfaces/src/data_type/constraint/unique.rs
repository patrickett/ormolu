use sqlx::{Database, Decode, prelude::Type};
use std::ops::Deref;

/// Unique constraint ensures that the data contained in this column is unique among all the rows in the table. The syntax is:
/// ```no_run,sql
/// CREATE TABLE products (
///     product_no integer UNIQUE,
///     name text,
///     price numeric
/// );
/// ```
/// And in Rust:
/// ```no_run,rust
/// #[derive(Table)]
/// pub struct Product {
///     product_no: Unique<i32>,
///     name: String,
///     price: bigdecimal::BigDecimal
/// }
/// ```
/// Adding a unique constraint will automatically create a unique B-tree index on the column or group of columns listed in the constraint.
///
/// A uniqueness restriction covering only some rows cannot be written as a unique constraint, but it is possible to enforce such a restriction by creating a unique partial index.
///
/// see: <https://www.postgresql.org/docs/17/ddl-constraints.html#DDL-CONSTRAINTS-UNIQUE-CONSTRAINTS>
#[repr(transparent)]
pub struct Unique<T>(T);

// TODO: To define a unique constraint for a group of columns, write it as a table constraint with the column names separated by commas:
// ```no_run,sql
// CREATE TABLE example (
//     a integer,
//     b integer,
//     c integer,
//     UNIQUE (a, c)
// );
// ```

impl<DB: Database, T> Type<DB> for Unique<T>
where
    String: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: Database, T> Decode<'r, DB> for Unique<T>
where
    String: Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        todo!()
    }
}

impl<T> AsRef<T> for Unique<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Unique<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<T> for Unique<T> {
    fn from(s: T) -> Self {
        Self(s)
    }
}
