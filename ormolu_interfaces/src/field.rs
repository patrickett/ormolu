pub use crate::query::QueryState;
use crate::query::Where;
use std::cell::RefCell;
use std::marker::PhantomData;
// use std::ops::Not;
use std::rc::Rc;

// NOTE: short circuiting logical and (&&) does not currently support overloading.
//
// See: https://doc.rust-lang.org/core/ops/
//  "Note that the && and || operators are currently not supported for overloading.
//  Due to their short circuiting nature, they require a different design from
//  traits for other operators like BitAnd. Designs for them are under discussion."
//
// This prevents from doing a something like:
//      `.filter(|order| !order.name.contains("jon") && order.id == 2)`

#[derive(Clone)]
pub struct Field<T> {
    pub field_name: &'static str,
    _marker: PhantomData<T>,
    pub state: Rc<RefCell<QueryState>>,
}

impl Field<String> {
    pub fn contains(&self, s: &str) -> bool {
        let mut state = self.state.borrow_mut();

        let ret = state.return_true();

        let mut w = Where::like(self.field_name, format!("%{s}%"));

        if !ret {
            w = w.wrap_not();
        }

        state.r#where.push(w);

        ret
    }
}

// name.contains, id==2, !name.contains
// true, true, true

// Allows `.filter(|customer| customer.id == 2)`
impl PartialEq<i32> for Field<i32> {
    fn eq(&self, other: &i32) -> bool {
        let mut state = self.state.borrow_mut();

        let ret = state.return_true();

        let mut w = Where::eq(self.field_name, other.to_string());

        if !ret {
            w = w.wrap_not();
        }

        state.r#where.push(w);

        ret
    }
}

impl PartialEq<String> for Field<String> {
    fn eq(&self, other: &String) -> bool {
        let mut state = self.state.borrow_mut();

        let ret = state.return_true();

        let mut w = Where::eq(self.field_name, other.to_string());

        if !ret {
            w = w.wrap_not();
        }

        state.r#where.push(w);

        ret
    }
}

impl PartialEq<String> for Field<&str> {
    fn eq(&self, other: &String) -> bool {
        let mut state = self.state.borrow_mut();

        let ret = state.return_true();

        let mut w = Where::eq(self.field_name, other.to_string());

        if !ret {
            w = w.wrap_not();
        }

        state.r#where.push(w);

        ret
    }
}

impl<T> Field<T> {
    pub fn new(field_name: &'static str, state: Rc<RefCell<QueryState>>) -> Self {
        Self {
            field_name,
            state,
            _marker: PhantomData,
        }
    }
}
