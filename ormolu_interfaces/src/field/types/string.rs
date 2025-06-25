pub use crate::query::QueryState;
use crate::{Col, query::Where};

impl Col<String> {
    pub fn contains(&self, s: &str) -> bool {
        let mut state = self.state.borrow_mut();
        let ret = state.return_true();
        let mut w = Where::like(self.field_name, format!("%{s}%"));
        if !ret {
            w = w.wrap_not();
        }

        state.clauses.push(w);
        ret
    }
}

impl PartialEq<String> for Col<String> {
    fn eq(&self, other: &String) -> bool {
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

impl PartialEq<String> for Col<&str> {
    fn eq(&self, other: &String) -> bool {
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
