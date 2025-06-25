use crate::{Table, query::Where};
use std::{cell::RefCell, rc::Rc};

#[derive(Default)]
pub struct FilterState {
    pub index: usize,
    /// This is the bool returned from each filter statement in the predicate
    pub returns: Vec<bool>,
    /// These are the actual conditions created
    pub clauses: Vec<Where>,
}

impl FilterState {
    pub fn return_true(&mut self) -> bool {
        self.index += 1;

        // if already exists in order then return it
        //
        // bool is flipped after running predicate outside
        if let Some(ret) = self.returns.get(self.index - 1) {
            return *ret;
        }

        self.returns.push(true);
        true
    }
}

/// Short circuiting logical and (&&) and or (||) does not currently support overloading.
///
/// See: <https://doc.rust-lang.org/core/ops/>
/// "Note that the && and || operators are currently not supported for overloading.
///  Due to their short circuiting nature, they require a different design from
///  traits for other operators like BitAnd. Designs for them are under discussion."
///
/// This prevents from doing a something like:
/// ```ignore
/// .filter(|order| !order.name.contains("jon") && order.id == 2)
/// ```
///
/// To get around this we store the bool for each statement within a filter
/// predicate and will flip return values to avoid short circuits done by negation.
/// Allowing the example above to reach the end.
#[derive(Default)]
pub struct FieldsFilter {
    pub state: Rc<RefCell<FilterState>>,
}

impl FieldsFilter {
    pub fn new<T, P>(predicate: P) -> FieldsFilter
    where
        T: Table,
        P: Fn(T::Proxy) -> bool,
    {
        let this = Self::default();

        let proxy = T::to_field_filter(this.state.clone());
        let mut positive_context = predicate(proxy);

        while !positive_context {
            {
                let mut state = this.state.borrow_mut();

                if let Some(b) = state.returns.pop() {
                    state.returns.push(!b);
                }

                state.clauses.clear();
                state.index = 0;
            } // drop borrow_mut

            let proxy2 = T::to_field_filter(this.state.clone());
            positive_context = predicate(proxy2);
        }

        this
    }
}
