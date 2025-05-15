pub use super::QueryState;
use crate::Gilded;
use std::{cell::RefCell, marker::PhantomData, rc::Rc};
mod field_filter;

pub struct QuerySet<T> {
    pub state: Rc<RefCell<QueryState>>,

    #[doc(hidden)]
    _marker: PhantomData<T>,
}

impl<T: Gilded> Default for QuerySet<T> {
    fn default() -> Self {
        Self {
            state: Rc::new(RefCell::new(QueryState::new(T::table_name(), T::fields()))),
            _marker: PhantomData,
        }
    }
}

// TODO: impl Filter from Iterator?

impl<T: Gilded> QuerySet<T> {
    // Filter internally saves the state of any query expressions
    // so they are all applied and we just keep the state internally
    pub fn filter<P>(self, predicate: P) -> Self
    where
        P: Fn(T::Proxy) -> bool,
    {
        // TODO: don't clone entire QueryState here
        let proxy = T::to_proxy(self.state.clone());
        let mut positive_context = predicate(proxy);
        println!("predicate: {}", &positive_context);

        while !positive_context {
            {
                let mut state = self.state.borrow_mut();

                if let Some(b) = state.filter_state.pop() {
                    state.filter_state.push(!b);
                }

                state.r#where.clear();
                state.current_index = 0;
            } // drop borrow_mut

            let proxy2 = T::to_proxy(self.state.clone());
            positive_context = predicate(proxy2);
            println!("predicate: {}", &positive_context);
        }

        {
            let a = self.state.borrow();
            println!("{a}");
        }

        self
    }
}

impl<T> std::fmt::Display for QuerySet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.state.borrow().to_string();
        write!(f, "{s}")
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

pub trait Findable
where
    Self: Sized,
{
    fn find() -> QuerySet<Self> {
        todo!()
    }

    fn find_or_create() -> QuerySet<Self> {
        todo!()
    }
}

impl<T> Fetch<T> for QuerySet<T> {}
