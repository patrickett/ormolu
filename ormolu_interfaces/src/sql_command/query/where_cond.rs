#[derive(Debug, Clone)]
pub enum WhereOp {
    EqualTo,
    GreaterThan,
    LessThan,
    GreaterThanOrEqualTo,
    LessThanOrEqualTo,
    Like,
    Not(Box<WhereOp>),
}

impl std::fmt::Display for WhereOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let a = match self {
            WhereOp::EqualTo => "=",
            WhereOp::GreaterThan => ">",
            WhereOp::LessThan => "<",
            WhereOp::GreaterThanOrEqualTo => ">=",
            WhereOp::LessThanOrEqualTo => "<=",
            WhereOp::Like => "LIKE",
            WhereOp::Not(nop) => match &**nop {
                WhereOp::EqualTo => "!=",
                WhereOp::GreaterThan => "<=",
                WhereOp::GreaterThanOrEqualTo => "<",
                WhereOp::LessThan => ">=",
                WhereOp::LessThanOrEqualTo => ">",
                WhereOp::Like => "NOT LIKE",
                WhereOp::Not(_) => {
                    unreachable!("wrap_not will unbox WhereOp::Not so this will never happen")
                }
            },
        };

        write!(f, "{a}")
    }
}

#[derive(Debug, Clone)]
pub struct Where {
    oper: WhereOp,
    column: &'static str,
    // its always going to be string since it will be converted to string anyways
    value: String,
}

impl std::fmt::Display for Where {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let col = &self.column;
        let value = &self.value;
        let op = self.oper.to_string();

        write!(f, "{col} {op} {value}")
    }
}

impl Where {
    pub fn like(column: &'static str, value: String) -> Self {
        Where {
            oper: WhereOp::Like,
            column,
            value,
        }
    }

    pub fn eq(column: &'static str, value: String) -> Self {
        Where {
            oper: WhereOp::EqualTo,
            column,
            value,
        }
    }

    pub fn neq(column: &'static str, value: String) -> Self {
        Where {
            oper: WhereOp::Not(Box::new(WhereOp::EqualTo)),
            column,
            value,
        }
    }

    pub fn wrap_not(mut self) -> Self {
        if let WhereOp::Not(op) = self.oper {
            self.oper = *op;
        } else {
            self.oper = WhereOp::Not(Box::new(self.oper));
        }
        self
    }
}
