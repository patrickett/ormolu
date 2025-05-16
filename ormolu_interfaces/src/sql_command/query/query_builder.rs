use super::Where;
use crate::Gilded;
use std::marker::PhantomData;

// TODO: phantom data where T::fields() && T::table_name

/// Data from which the actual sql query string will be built
pub struct QueryState<T> {
    // list of columns - TODO: by default should be all columns instead of *
    pub select: Vec<String>,
    // table name selecting from
    pub from: &'static str,
    // JOIN
    pub r#where: Vec<Where>,
    // ORDER BY
    // GROUP BY
    // HAVING
    pub limit: Option<i64>,
    pub offset: Option<i64>,

    _marker: PhantomData<T>,
}

impl<T: Gilded> Default for QueryState<T> {
    fn default() -> Self {
        Self {
            from: T::table_name(),
            // need to get fields from type T
            select: T::fields(),
            r#where: Vec::new(),
            limit: None,
            offset: None,
            _marker: PhantomData,
        }
    }
}

impl<T> std::fmt::Display for QueryState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fields = if self.select.is_empty() {
            // TODO: QueryState<T: Gilded> T::fields()
            "*".to_string()
        } else {
            self.select.join(", ")
        };

        let select = format!("SELECT {fields}");

        let from = format!(" FROM {}", self.from);

        let where_cond = if self.r#where.is_empty() {
            String::new()
        } else {
            let where_conditions = self
                .r#where
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

        write!(f, "{select}{from}{where_cond}{limit}{offset}")
    }
}
