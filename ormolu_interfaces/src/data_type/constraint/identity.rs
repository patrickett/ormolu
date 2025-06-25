use std::ops::Deref;

/// An identity column is a special column that is generated automatically from an implicit sequence.
///
/// It can be used to generate key values.
///
/// see: <https://www.postgresql.org/docs/current/ddl-identity-columns.html#DDL-IDENTITY-COLUMNS>
#[repr(transparent)]
pub struct Identity<T>(T);

impl<T> Deref for Identity<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
