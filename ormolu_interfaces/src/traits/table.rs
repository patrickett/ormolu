use super::DatabaseObject;
use crate::FilterState;
use crate::OrmoluError;
use std::{cell::RefCell, rc::Rc};

/// Represents a database table.
///
/// This trait is implemented by types that correspond to tables in a database.
pub trait Table
where
    Self: DatabaseObject + Filterable + HasFields + Sized,
{
}

/// Provides methods for working with keys of entities.
///
/// This trait enables getting an entity from a key and accessing the entity type associated with the key.
pub trait Key<Entity, T> {
    /// Asynchronously retrieves an entity by its key.
    ///
    /// Returns `Ok(Some(entity))` if found, `Ok(None)` if not found, or an error.
    fn get_entity(&self) -> impl Future<Output = Result<Option<Entity>, OrmoluError>>;
}

/// Provides methods for working with primary keys in the context of a table.
///
/// This trait enables accessing the primary key value and retrieving records by their primary key.
pub trait HasPrimaryKey<T>
where
    Self: Sized,
{
    /// Returns a reference to the primary key of this record.
    fn primary_key(&self) -> &T;

    /// Asynchronously retrieves a record by its primary key.
    ///
    /// Returns `Ok(Some(record))` if found, `Ok(None)` if not found, or an error.
    fn get_by_primary_key(key: &T) -> impl Future<Output = Result<Option<Self>, OrmoluError>>;
}

/// This uses some macro reflection to create a field mapping/lookup from the
/// Rust structs' field to the actual database column name.
pub trait HasFields {
    /// Returns the static map of field names to database column names.
    fn field_map() -> &'static phf::Map<&'static str, &'static str>;

    /// Returns a list of all the database columns.
    fn database_columns() -> &'static [&'static str];

    #[inline]
    fn get_db_column_name(key: &'static str) -> &'static str {
        Self::field_map().get(key).copied().unwrap_or(key)
    }

    /// Returns the ordinal position of the database column with a matching name.
    fn ordinal(column: &'static str) -> Option<usize> {
        Option::map(
            Self::database_columns().iter().position(|c| *c == column),
            |c| c + 1,
        )
    }

    /// Returns the column name for the provided ordinal position.
    fn column(ordinal: usize) -> Option<&'static str> {
        let index = ordinal - 1;
        Option::map(Self::database_columns().get(index), |c| *c)
    }
}

/// Create a Proxy version of the struct so we can interact with it as if
/// we were comparing real values but under the hood it actually is building
/// a query.
///
/// This wraps each field in a struct with a `Field<T>` where field manages the
/// query state
pub trait Filterable {
    type Proxy;

    /// Converts to a proxy object for constructing filters.
    fn to_field_filter(state: Rc<RefCell<FilterState>>) -> Self::Proxy;
}

pub trait Selectable
where
    Self::Select: Default,
{
    type Select;

    fn select() -> Self::Select;
}
