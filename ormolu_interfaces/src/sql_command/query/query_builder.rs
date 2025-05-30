use super::{Command, Where};
use crate::Gilded;
use std::marker::PhantomData;

pub struct QueryState<T> {
    pub command: Command,
    pub table: &'static str,
    // JOIN
    pub where_conditions: Vec<Where>,
    // ORDER BY
    // GROUP BY
    // HAVING
    pub limit: Option<i64>,
    pub offset: Option<i64>,

    _marker: PhantomData<T>,
}

impl<T: Gilded> QueryState<T> {
    pub fn new_select() -> Self {
        Self {
            command: Command::Select {
                fields: T::fields(),
            },
            table: T::table_name(),
            where_conditions: Vec::new(),
            limit: None,
            offset: None,
            _marker: PhantomData,
        }
    }
}

impl<T> std::fmt::Display for QueryState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let command = match &self.command {
            Command::Select { fields } => {
                format!("SELECT {} FROM {}", fields.join(", "), self.table)
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
