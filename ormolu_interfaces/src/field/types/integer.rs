use crate::{Field, query::Where};

impl PartialEq<i32> for Field<i32> {
    fn eq(&self, other: &i32) -> bool {
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
