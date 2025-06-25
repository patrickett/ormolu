//! A data type is an attribute that specifies the type of data that can be stored in a column of a database table.
//!
//! Data types determine the kind of operations that can be performed on the data and how much space it occupies in the database

pub mod character;
pub use character::*;

pub mod constraint;
pub use constraint::*;

pub mod numeric;
pub use numeric::*;
