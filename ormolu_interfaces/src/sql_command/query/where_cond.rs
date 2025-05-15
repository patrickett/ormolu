#[derive(Debug, Clone)]
pub enum WhereOp {
    /// =
    EqualTo,
    /// !=
    NotEqualTo,
    /// >
    GreaterThan,
    /// <
    LessThan,
    /// >=
    GreaterThanOrEqualTo,
    /// <=
    LessThanOrEqualTo,
    Like,

    Not(Box<WhereOp>),
}

impl std::fmt::Display for WhereOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: PERF some const stuff can likely be done here
        let a = match self {
            WhereOp::EqualTo => "=".into(),
            WhereOp::NotEqualTo => "!=".into(),
            WhereOp::GreaterThan => ">".into(),
            WhereOp::LessThan => "<".into(),
            WhereOp::GreaterThanOrEqualTo => ">=".into(),
            WhereOp::LessThanOrEqualTo => "<=".into(),
            WhereOp::Like => "LIKE".into(),
            WhereOp::Not(op) => format!("NOT {op}"),
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
            oper: WhereOp::NotEqualTo,
            column,
            value,
        }
    }

    pub fn wrap_not(mut self) -> Self {
        let old_oper = self.oper.clone();
        self.oper = WhereOp::Not(Box::new(old_oper));
        self
    }
}
