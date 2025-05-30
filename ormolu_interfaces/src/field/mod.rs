pub mod filter;
pub mod types;

pub use crate::query::QueryState;
pub use filter::*;
pub use types::*;

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct Field<T> {
    pub field_name: &'static str,
    _marker: PhantomData<T>,
    pub state: Rc<RefCell<FilterState>>,
}

impl<T> Field<T> {
    pub fn new(field_name: &'static str, state: Rc<RefCell<FilterState>>) -> Self {
        Self {
            field_name,
            state,
            _marker: PhantomData,
        }
    }
}
