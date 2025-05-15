pub use ormolu_interfaces::*;
pub use ormolu_macros::*;

mod test;

//// Internal concept for what is basically a reference to a SQL Table.
// #[derive(Default)]
// pub struct Collection<T> {
//     phantom: PhantomData<T>,
// }

// impl<T> Collection<T> {
//     pub fn new() -> Self {
//         Self {
//             phantom: PhantomData,
//         }
//     }
// }
