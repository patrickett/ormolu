use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

/// A database object is any defined object in a database that is used to store or manipulate data.
///
/// This trait serves as a marker for objects that can interact with a database.
pub trait DatabaseObject
where
    Self: HasQualifiedName + GetConnectionPool,
{
}

/// Represents an object that has both a schema name and a name, providing a qualified name.
///
/// This trait combines the functionality of [`HasSchemaName`] and [`HasName`] to provide a fully-qualified
/// name for database objects.
pub trait HasQualifiedName: HasSchemaName + HasObjectName {
    /// Returns the fully qualified name of this object as a static string slice.
    ///
    /// The qualified name typically consists of schema name and object name, usually in the format:
    /// `schema_name.object_name`.
    fn qualified_name() -> &'static str;
}

/// Represents an entity that has a schema name within a database.
///
/// This trait provides a way to retrieve the schema name for database objects.
pub trait HasSchemaName {
    /// Returns the schema name of this object as a static string slice.
    ///
    /// The schema name typically represents the namespace or category under which the
    /// object is categorized in the database.
    fn schema_name() -> &'static str;
}

/// Represents an entity that has a name within a database.
///
/// This trait provides a way to retrieve the name for database objects.
pub trait HasObjectName {
    /// Returns the name of this object as a static string slice.
    ///
    /// The name typically represents the unique identifier or title of the
    /// object within its schema.
    fn object_name() -> &'static str;
}

// TODO: replace with dep injection in header #[gild(pool = get_cust_pool)]
/// This trait defines how any database object can get its own ConnectionPool
pub trait GetConnectionPool {
    fn get_connection_pool() -> impl Future<Output = Pool<Postgres>> {
        async {
            dotenvy::dotenv().ok();

            let database_url =
                std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

            PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await
                .expect("Failed to create pool")
        }
    }
}
