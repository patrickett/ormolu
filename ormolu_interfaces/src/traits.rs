use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::FilterState;
use std::{cell::RefCell, env, rc::Rc};

/// [`Gilded`] is a supertrait to ensure all the necessary component traits
/// are satisfied.
pub trait Gilded
where
    Self: Filterable + Selectable + HasTableName + HasFields + Sized,
{
    #[cfg(debug_assertions)]
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
pub trait Filterable {
    type Proxy;

    fn to_field_filter(state: Rc<RefCell<FilterState>>) -> Self::Proxy;
}

pub trait Selectable
where
    Self::Select: Default,
{
    type Select;

    fn select() -> Self::Select;
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
    fn fields() -> &'static [&'static str];
}

// TODO: this can likely be improved
// The idea is we don't want to have to specify the .fetch(&pool) every damn
// call site we want to talk to the database. This trait makes it so the
// model itself can at least get a connection inefficently remove the need
// for the constant passing of the pool parameter. It should also allow custom
// impl for types
// pub trait GetPool {
//     fn get_pool() -> impl Future<Output = Pool<Postgres>> + Send {
//         async {
//             let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

//             PgPoolOptions::new()
//                 .max_connections(5)
//                 .connect(&db_url)
//                 .await
//                 .expect("Failed to connect to database")
//         }
//     }
// }
