extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, ItemStruct};

#[proc_macro_attribute]
pub fn ormolu(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("attr: \"{}\"", attr.to_string());
    println!("item: \"{}\"", item.to_string());
    item
}

// begin ormolu
// #[ormolu(table_name = "user")]
// struct User {
//     #[ormolu(primary_key, column_name = "id")]
//     id: i32,
// }
// end ormolu -- Generated 2022-09-27 18:00:00.000
