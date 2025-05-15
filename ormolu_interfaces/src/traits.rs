use crate::query::QueryState;
use std::{cell::RefCell, rc::Rc};

pub trait Gilded
where
    Self: ToProxy + HasTableName + HasFields,
{
    fn is_gilded() -> bool {
        true
    }
}

/// Create a Proxy version of the struct so we can interact with it as if
/// we were comparing real values but under the hood it actually is building
/// a query.
///
/// This wraps each field in a struct with a `Field<T>` where field manages the
/// query state
pub trait ToProxy {
    type Proxy;

    fn to_proxy(state: Rc<RefCell<QueryState>>) -> Self::Proxy;
}

pub trait HasTableName {
    fn table_name() -> &'static str;
}

pub trait HasSchemaName {
    fn schema_name() -> &'static str;
}

pub trait HasPrimaryKey<T> {
    fn primary_key(&self) -> &T;
    fn primary_key_field_name() -> &'static str;
}

#[allow(dead_code)]
pub trait UniqueConstraint {
    // unsure what to do here
}

pub trait HasFields {
    // TODO: no alloc here
    // fn fields() -> &'static [&'static str];
    fn fields() -> Vec<String>;
}
