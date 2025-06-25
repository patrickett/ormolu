/// A wrapper type representing a PostgreSQL char with a fixed length.
///
/// This struct is a transparent wrapper around a byte array of a specified constant size `N`,
/// designed to represent PostgreSQL character types.
#[repr(transparent)]
pub struct Char<const N: usize>([u8; N]);
