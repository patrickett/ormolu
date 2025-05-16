pub use super::QueryState;
use crate::*;

#[derive(Default)]
pub struct QuerySet<T: Gilded> {
    state: QueryState<T>,
}

// TODO: impl Filter from Iterator?
impl<T: Gilded> QuerySet<T> {
    pub fn filter<P>(mut self, predicate: P) -> Self
    where
        P: Fn(T::Proxy) -> bool,
    {
        let filter = FieldsFilter::new::<T, P>(predicate);
        self.state
            .r#where
            .append(&mut filter.state.borrow_mut().clauses);

        self
    }
}

impl<T: Gilded> std::fmt::Display for QuerySet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.state)
    }
}

pub trait Fetch<T> {
    fn all(&self) -> impl Future<Output = Result<Vec<T>, sqlx::Error>> {
        async move { todo!() }
    }

    fn first(&self) -> impl Future<Output = Result<Option<T>, sqlx::Error>> {
        async move { todo!() }
    }

    fn last(&self) -> impl Future<Output = Result<Option<T>, sqlx::Error>> {
        async move { todo!() }
    }
}

// impl<T> Fetch<T> for QuerySet<T> {}
