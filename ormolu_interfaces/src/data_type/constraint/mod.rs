//! A constraint is a rule applied to a column or a set of columns in a database
//! table that restricts the type of data that can be inserted, updated, or deleted.
//!
//! Constraints help maintain the integrity and accuracy of the data within the database.

mod foreign_key;
pub use foreign_key::*;

mod primary_key;
pub use primary_key::*;

mod unique;
pub use unique::*;

mod identity;
pub use identity::*;

mod check;
pub use check::*;
