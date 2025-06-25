pub use super::QueryState;
use crate::*;
use futures::stream::Stream;
use std::pin::*;
use std::task::*;

/**
Internally, a QuerySet can be constructed, filtered, sliced, and generally
passed around without actually hitting the database. No database activity
actually occurs until you do something to evaluate the queryset.

You can evaluate a QuerySet in the following ways:

- Iteration. A QuerySet is iterable, and it executes its database query the first time you iterate over it. For example, this will print the headline of all entries in the database:
```rust,ignore
    let customer = Customer::get_by_id(1).await?;
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
pub struct QuerySet<T: Table> {
    // TODO: merge QueryState into queryset
    state: QueryState<T>,
}

impl<T: Table> QuerySet<T> {
    pub fn new(state: QueryState<T>) -> Self {
        Self { state }
    }
}

impl<T: Table> std::fmt::Display for QuerySet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.state)
    }
}
// TODO: methods like this
// ```rust,ignore
//     while let Some(order) = orders.all(&db) {
//         println!("ORDER: {}", order);
//     }
// ```
// so you can QuerySet<T>::all() which is basically a into_async_iter

// impl<T: Table> IntoIterator for QuerySet<T> {
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

// impl<T: Table> IntoAsyncIterator for QuerySet<T> {
//     type Item = T;
//     type IntoAsyncIter = AsyncIterator<Item = Self::Item>;

//     fn into_async_iter(self) -> Self::IntoAsyncIter {
//         todo!()
//         // self.0.into_iter()
//     }
// }

// impl<T> Iterator for QuerySet<T>
// where
//     T: Table,
// {
//     type Item = ;

//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
// }

impl<T> Stream for QuerySet<T>
where
    T: Table,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        todo!()
    }
}

// async iterators and queryset are good fit since they are lazy and do nothing
// unless polled

// TODO: impl Filter from Iterator?
// AsyncIterator?
impl<T: Table> QuerySet<T> {
    /// Returns a QuerySet containing WHERE clauses derived from the filters.
    ///
    /// Filters are joined via AND in the underlying SQL statement.
    ///
    /// If you need to execute more complex queries
    /// (for example, queries with OR statements), you can use the `.or()` syntax.
    // NOTE: this Shadows Iterator::filter
    // TODO: Change TFilter to AndFilter<T> with deref
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
