mod database_object;
mod derive;
mod table_metadata;
mod utils;

use proc_macro::TokenStream;
use syn::*;

// TODO: Use the filter at compile time, since we know it maps to actual db tables
// the filter just creates the sql so there is no reason it has to run at
// runtime we just need something to actually create the query before the filter
// logic runs. macro will do that
#[proc_macro]
pub fn create_select(input: TokenStream) -> TokenStream {
    // Parse the input tokens into an identifier
    let struct_name = parse_macro_input!(input as Ident);

    // Generate the struct definition
    let expanded = quote::quote! {
        struct #struct_name {
            pub field: String,
        }
    };

    // Convert the generated code back to TokenStream
    TokenStream::from(expanded)
}

#[proc_macro_derive(Table, attributes(object, schema, name, gild))]
pub fn derive_table(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    derive::table(&derive_input)
}

#[proc_macro_derive(StoredProcedure, attributes(object, schema, name, gild))]
pub fn derive_stored_procedure(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    derive::stored_procedure(&derive_input)
}

#[proc_macro_derive(View, attributes(object, schema, name, gild))]
pub fn derive_view(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    derive::view(&derive_input)
}

#[proc_macro_derive(DatabaseObject, attributes(name, schema, object))]
pub fn derive_database_object(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    derive::database_object(&derive_input)
}

#[proc_macro_derive(Selectable)]
pub fn derive_selectable(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    derive::selectable(&derive_input)
}

#[proc_macro_derive(Filterable)]
pub fn derive_filterable(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    derive::filterable(&derive_input)
}
