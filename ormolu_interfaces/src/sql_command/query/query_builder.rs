use super::Where;

// TODO: phantom data where T::fields() && T::table_name

/// Data from which the actual sql query string will be built
#[derive(Clone)]
pub struct QueryState {
    pub current_index: usize,
    pub filter_state: Vec<bool>,

    // TODO: move

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
}

impl QueryState {
    pub fn return_true(&mut self) -> bool {
        self.current_index += 1;

        // if already exists in order then return it
        //
        // bool is flipped after running predicate outside
        if let Some(ret) = self.filter_state.get(self.current_index - 1) {
            return *ret;
        }

        self.filter_state.push(true);
        true
    }

    pub fn new(table_name: &'static str, fields: Vec<String>) -> Self {
        Self {
            current_index: 0,
            filter_state: Vec::new(),

            from: table_name,
            // need to get fields from type T
            select: fields,
            r#where: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    /// Returns true if the query can return more than 1 record
    pub fn is_many(&self) -> bool {
        if let Some(limit) = self.limit {
            limit > 1
        } else {
            true
        }
    }
}

impl std::fmt::Display for QueryState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fields = if self.select.is_empty() {
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
