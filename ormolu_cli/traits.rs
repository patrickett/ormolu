struct User {
    id: usize,
    name: String,
}

struct UserTable {
    connection: String,
}

struct Database {
    connection: String,
}

impl Database {
    pub fn new(conn: impl Into<String>) -> Self {
        Self {
            connection: conn.into(),
        }
    }

    pub fn user(&self) -> UserTable {
        todo!()
    }
}

// select! [id, name]

pub trait Select<T> {
    fn select<F>(&self, selection: F) -> Vec<F>;
}

pub trait Where<T> {}

pub trait Query<T> {
    fn find_first(where_cond: impl Into<String>, connection: impl Into<String>) -> Option<T>;
    fn find_many(where_cond: impl Into<String>, connection: impl Into<String>) -> Vec<T>;
}

/// A managed query is the same as `Query<T>` except that it has the implicit context
/// of how to interact with the database. Since the database has already been instantiated.
pub trait ManagedQuery<T> {
    fn find_first(&self, where_cond: impl Into<String>) -> Option<T>;
    fn find_many(&self, where_cond: impl Into<String>) -> Vec<T>;
}

impl Query<User> for User {
    fn find_first(where_cond: impl Into<String>, connection: impl Into<String>) -> Option<User> {
        todo!()
    }

    fn find_many(where_cond: impl Into<String>, connection: impl Into<String>) -> Vec<User> {
        todo!()
    }
}

impl ManagedQuery<User> for UserTable {
    fn find_first(&self, where_cond: impl Into<String>) -> Option<User> {
        User::find_first(where_cond, &self.connection)
    }

    fn find_many(&self, where_cond: impl Into<String>) -> Vec<User> {
        User::find_many(where_cond, &self.connection)
    }
}

// DatabaseConnection ($db_name)
//      DatabaseTable ($table_name)
//          DatabaseRow ($)

// generate database struct, which is a struct that can hold connection scheme
// and can be shared via imports and it keeps its own thing to allow nice db_name.user().find_first(...)

// we should also allow an alternative User::find_first(where_cond, &mut connection)

fn example() {
    let conn = "conn";
    let db = Database::new(conn);
    let user = db.user().find_first("where_cond");

    let user2 = User::find_first("where_cond", conn);

    // User::select(select![name, id])
}

// struct Query<T> {
//     // defaults to *
//     select: Option<Select<T>>,
//     // from: T (we know the table/row because of T)
//     join: Join<T>,

//     // WHERE clause
//     filter: Filter<T>,
//     order_by: OrderBy<T>,
//     group_by: GroupBy<T>,
//     having: Option<Having<T>>,
// }

pub struct NotEqual<T> {
    not_equal: T,
}

pub struct Equal<T> {
    equal: T,
}

fn ne<T>(not_equal: T) -> NotEqual<T> {
    NotEqual { not_equal }
}

fn eq<T>(equal: T) -> Equal<T> {
    Equal { equal }
}

pub enum Compare<T> {
    Equal(Equal<T>),
    NotEqual(NotEqual<T>),
}

#[derive(Default)]
pub struct QueryBuilder {
    // defaults to *
    // select: Option<Select<T>>,
    // from: T (we know the table/row because of T)
    // join: Option<Join<T>>,

    // WHERE clause
    // filter: Option<Filter<T>>,
    // order_by: Option<OrderBy<T>>,
    // group_by: Option<GroupBy<T>>,
    // having: Option<Having<T>>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn select(&self) -> Self {
        todo!()
    }
    pub fn join(&self) -> Self {
        todo!()
    }
    pub fn filter(&self) -> Self {
        todo!()
    }
    pub fn order_by(&self) -> Self {
        todo!()
    }
    pub fn group_by(&self) -> Self {
        todo!()
    }
    pub fn having(&self) -> Self {
        todo!()
    }
}
// have normal query builder with very minimal typing
// Select::from("user").and_where(...)

// but allow dynamically creating from db types to implement it as well
// User::select(["name", "id"]).and_where(...)
macro_rules! select_fields {
    ($instance:expr, { $($field:ident),+ }) => {
        {
            #[derive(Debug)]
            struct SelectedFields {
                $(pub $field: String,)+
            }

            SelectedFields {
                $(
                    $field: $instance.$field.clone(),
                )+
            }
        }
    };
}

fn ex() {
    let person = User {
        name: "John Doe".to_string(),
        id: 30,
        // email: "john.doe@example.com".to_string(),
        // address: "123 Elm Street".to_string(),
    };

    let f = select_fields!(person, { name });

    // let qq = Qry::select().filter().group_by()
    let qb = QueryBuilder::new();

    // qb.select().filter()

    // let q = qry! {
    //     select: ["name", "id"],
    //     filter: {
    //         id: ne(4), // NotEqual<usize>
    //     },
    //     order_by: desc("name") // Desc<str>
    // };
    // this should return a generic, that can be used to check against the type of the row

    // User::query(q)
}

use quote::format_ident;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

fn has_matching_fields<T: 'static>(field_names: Vec<&str>) -> bool {
    // Get the type name of T
    let type_name = std::any::type_name::<T>();

    // Parse the type to get the struct fields
    let input: DeriveInput = syn::parse_str(type_name).expect("Failed to parse type");

    if let Data::Struct(data) = input.data {
        if let Fields::Named(fields) = data.fields {
            // Collect all the field names in the struct
            let struct_field_names: Vec<String> = fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap().to_string())
                .collect();

            // Check if all the field names in the input vector exist in the struct
            return field_names
                .iter()
                .all(|&name| struct_field_names.contains(&name.to_string()));
        }
    }
    false
}

// Example usage with a test struct
#[derive(Debug)]
struct TestStruct {
    foo: i32,
    bar: String,
    baz: bool,
}

fn main() {
    let field_names = vec!["foo", "bar"];
    let result = has_matching_fields::<TestStruct>(field_names);
    println!("Result: {}", result); // Output: Result: true

    let field_names = vec!["foo", "qux"];
    let result = has_matching_fields::<TestStruct>(field_names);
    println!("Result: {}", result); // Output: Result: false
}
