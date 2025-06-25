use super::Where;
use crate::{Command, Table};
use std::marker::PhantomData;

// Ref is an implementation detail you can ignore
// This is a reference to a field on a struct
// Where E is the 'Entity' and C is the ordinal position of the field
pub struct Ref<E, const C: usize> {
    _entity: PhantomData<E>,
}

struct Col<T, E, const C: usize> {
    _data: PhantomData<T>,
    _ref: Ref<E, C>,
}

pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

pub struct ColRef {
    schema: Option<String>,
    object: String,
    column: String,
}

pub struct Join {
    join_type: JoinType,
    on: [ColRef; 2],
}

pub struct GroupBy {
    columns: Vec<ColRef>,
}

pub struct OrderBy {
    columns: Vec<ColRef>,
}

impl Join {
    fn inner(dbo1: ColRef, dbo2: ColRef) -> Self {
        Self {
            join_type: JoinType::Inner,
            on: [dbo1, dbo2],
        }
    }

    fn left(dbo1: ColRef, dbo2: ColRef) -> Self {
        Self {
            join_type: JoinType::Left,
            on: [dbo1, dbo2],
        }
    }

    fn right(dbo1: ColRef, dbo2: ColRef) -> Self {
        Self {
            join_type: JoinType::Right,
            on: [dbo1, dbo2],
        }
    }

    fn full(dbo1: ColRef, dbo2: ColRef) -> Self {
        Self {
            join_type: JoinType::Full,
            on: [dbo1, dbo2],
        }
    }

    fn cross(dbo1: ColRef, dbo2: ColRef) -> Self {
        Self {
            join_type: JoinType::Cross,
            on: [dbo1, dbo2],
        }
    }
}

/// The HAVING clause was added to SQL because the WHERE keyword cannot be used with aggregate functions.
pub struct Having {}

pub struct QueryState<T> {
    pub command: Command,
    pub table: &'static str,
    pub where_conditions: Vec<Where>,
    pub joins: Vec<Join>,
    pub order_by: Option<OrderBy>,
    pub group_by: Option<GroupBy>,
    pub having: Option<Having>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    _table: PhantomData<T>,
}

impl<T: Table> QueryState<T> {
    pub fn new_select() -> Self {
        Self {
            command: Command::Select {
                columns: T::database_columns(),
            },
            table: T::object_name(),
            where_conditions: Vec::new(),
            joins: Vec::new(),
            limit: None,
            offset: None,
            _table: PhantomData,
            order_by: None,
            group_by: None,
            having: None,
        }
    }
}

impl<T> std::fmt::Display for QueryState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let command = match &self.command {
            Command::Select { columns } => {
                format!("SELECT {} FROM {}", columns.join(", "), self.table)
            }
            Command::Delete => {
                format!("DELETE FROM {}", self.table)
            }
            _ => todo!(),
        };

        let where_conditions = if self.where_conditions.is_empty() {
            String::new()
        } else {
            let where_conditions = self
                .where_conditions
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join(" AND ");

            format!(" WHERE {where_conditions}")
        };

        let limit = if let Some(lmt) = self.limit {
            format!(" LIMIT {lmt}")
        } else {
            String::new()
        };

        let offset = if let Some(oft) = self.offset {
            format!(" OFFSET {oft}")
        } else {
            String::new()
        };

        write!(f, "{command}{where_conditions}{limit}{offset};")
    }
}
