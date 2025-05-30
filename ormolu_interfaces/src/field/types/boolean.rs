// use std::ops::Deref;

use std::ops::Not;

use crate::{Field, query::Where};

impl PartialEq<bool> for Field<bool> {
    fn eq(&self, other: &bool) -> bool {
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

impl Not for Field<bool> {
    type Output = bool;

    fn not(self) -> Self::Output {
        // let mut field = self.clone();
        let mut state = self.state.borrow_mut();
        let ret = state.return_true();
        let mut w = Where::eq(self.field_name, (!ret).to_string());
        if !ret {
            w = w.wrap_not();
        }

        state.clauses.push(w);
        ret
    }
}

// impl Deref for Field<bool> {
//     type Target = bool;

//     fn deref(&self) -> &Self::Target {
//         &true
//     }
// }
