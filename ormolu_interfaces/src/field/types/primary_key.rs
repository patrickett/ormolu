use std::fmt::Display;

use crate::{Col, PrimaryKey, query::Where};

impl<E, T: Display> PartialEq<PrimaryKey<E, T>> for Col<PrimaryKey<E, T>> {
    fn eq(&self, other: &PrimaryKey<E, T>) -> bool {
        let mut state = self.state.borrow_mut();
        let ret = state.return_true();
        let mut w = Where::eq(self.field_name, other.to_string());
        if !ret {
            w = w.wrap_not();
        }

        state.clauses.push(w);
        ret
    }
}
