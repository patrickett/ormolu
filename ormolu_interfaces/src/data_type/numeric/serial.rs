/// PostgreSQL `SERIAL` is a pseudo-type used to create auto-incrementing integer
/// columns, typically for primary keys.
///
/// It automatically generates a sequence of numbers for each new row,
/// simplifying the process of assigning unique identifiers.
///
/// See: <https://www.postgresql.org/docs/17/datatype-numeric.html#DATATYPE-SERIAL>
///
/// **Developer note**: The serial types have some weird behaviors that make schema, dependency, and permission management unnecessarily cumbersome.
///
/// see: <https://wiki.postgresql.org/wiki/Don%27t_Do_This#Don.27t_use_serial>
pub struct Serial(i32);
