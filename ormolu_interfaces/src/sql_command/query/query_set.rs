// use std::async_iter::{AsyncIterator, IntoAsyncIterator};

pub use super::QueryState;
use crate::*;

/**
Internally, a QuerySet can be constructed, filtered, sliced, and generally
passed around without actually hitting the database. No database activity
actually occurs until you do something to evaluate the queryset.

You can evaluate a QuerySet in the following ways:

- Iteration. A QuerySet is iterable, and it executes its database query the first time you iterate over it. For example, this will print the headline of all entries in the database:
```rust,ignore
    let customer = Customer::find_by_id(1).await?;
    let orders: QuerySet<Order> = customer::orders();
    let real_orders = orders.filter(|o| !o.test);

    for order in real_orders { // This is when the query will be executed
        println!("ORDER: {}", order);
    }
```

- Asynchronous iteration. A QuerySet can also be iterated over using async for:
```rust,ignore
    while let Some(order) = orders.next().await {
        println!("ORDER: {}", order);
    }
```
*/
pub struct QuerySet<T: Gilded> {
    state: QueryState<T>,
}

impl<T: Gilded> QuerySet<T> {
    pub fn new(state: QueryState<T>) -> Self {
        Self { state }
    }
}

impl<T: Gilded> std::fmt::Display for QuerySet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.state)
    }
}

// impl<T: Gilded> IntoIterator for QuerySet<T> {
//     type Item = T;
//     type IntoIter = std::vec::IntoIter<Self::Item>;

//     // TODO: doc comment as this should actually evaluate the QuerySet
//     // and fetch the data from the database
//     // replace default doc comment with some info about this
//     fn into_iter(self) -> Self::IntoIter {
//         todo!()
//         // self.0.into_iter()
//     }
// }

// impl<T: Gilded> IntoAsyncIterator for QuerySet<T> {
//     type Item = T;
//     type IntoAsyncIter = AsyncIterator<Item = Self::Item>;

//     fn into_async_iter(self) -> Self::IntoAsyncIter {
//         todo!()
//         // self.0.into_iter()
//     }
// }

impl<T: Gilded> Iterator for QuerySet<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // let sql = self.state.to_string();
        // let f = sqlx::query(&sql).fetch_all(executor);
        todo!()
    }
}

// TODO: impl Filter from Iterator?
// AsyncIterator?
impl<T: Gilded> QuerySet<T> {
    /// Returns a QuerySet containing WHERE clauses derived from the filters.
    ///
    /// Filters are joined via AND in the underlying SQL statement.
    ///
    /// If you need to execute more complex queries
    /// (for example, queries with OR statements), you can use the `.or()` syntax.
    // NOTE: this Shadows Iterator::filter
    pub fn filter<P>(mut self, predicate: P) -> Self
    where
        P: Fn(T::Proxy) -> bool,
    {
        let filter = FieldsFilter::new::<T, P>(predicate);
        self.state
            .where_conditions
            .append(&mut filter.state.borrow_mut().clauses);

        self
    }

    // TODO: remove this - figure out how to check if || in statement
    /// Returns a QuerySet containing WHERE clauses derived from the filters.
    ///
    /// Filters are joined via OR in the underlying SQL statement.
    pub fn filter_or<P>(mut self, predicate: P) -> Self
    where
        P: Fn(T::Proxy) -> bool,
    {
        let filter = FieldsFilter::new::<T, P>(predicate);
        self.state
            .where_conditions
            .append(&mut filter.state.borrow_mut().clauses);

        self
    }
}
