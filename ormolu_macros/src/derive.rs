use crate::{database_object::expand_derive_database_object, table_metadata::expand_derive_table};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use sqlx_macros_core::derives::expand_derive_from_row;
use syn::*;

pub fn database_object(derive_input: &DeriveInput) -> TokenStream {
    match expand_derive_database_object(derive_input) {
        Ok(ts) => ts.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

pub fn table(derive_input: &DeriveInput) -> TokenStream {
    let filterable: TokenStream2 = filterable(derive_input).into();
    let selectable: TokenStream2 = selectable(derive_input).into();

    let dbo_impl = match expand_derive_database_object(derive_input) {
        Ok(ts) => ts,
        Err(e) => return e.into_compile_error().into(),
    };

    let from_row = match expand_derive_from_row(derive_input) {
        Ok(ts) => ts,
        Err(e) => return e.into_compile_error().into(),
    };

    let table_metadata = match expand_derive_table(derive_input) {
        Ok(t) => t,
        Err(err) => return err.into_compile_error().into(),
    };

    let table = table_metadata.to_token_stream();

    quote! {
        #dbo_impl
        #from_row
        #filterable
        #selectable
        #table
    }
    .into()
}

pub fn stored_procedure(derive_input: &DeriveInput) -> TokenStream {
    let dbo_impl = match expand_derive_database_object(derive_input) {
        Ok(ts) => ts,
        Err(e) => return e.into_compile_error().into(),
    };

    // TODO: parse top attributes for schema and name
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();

    // Match on the data to extract field names and types
    if let Data::Struct(data_struct) = &derive_input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            for field in &fields.named {
                // Get the field name and type
                let field_name = field.ident.clone();
                let field_type = field.ty.clone();

                field_names.push(field_name);
                field_types.push(field_type);
            }
        }
    }
    let struct_name = &derive_input.ident;

    quote! {
        #dbo_impl

        impl ormolu_interfaces::StoredProcedure for #struct_name {
            fn execute<T>(&self) -> Result<T, String> {
                todo!()
            }
        }

        impl #struct_name {
            fn exec<T>(#(#field_names: #field_types),*) -> Result<T, String> {
                let params = #struct_name {#(#field_names),*};
                params.execute()
            }
        }
    }
    .into()
}

pub fn view(derive_input: &DeriveInput) -> TokenStream {
    let dbo_impl = match expand_derive_database_object(derive_input) {
        Ok(ts) => ts,
        Err(e) => return e.into_compile_error().into(),
    };

    quote! {
        #dbo_impl
    }
    .into()
}

pub fn filterable(derive_input: &DeriveInput) -> TokenStream {
    let struct_name = &derive_input.ident;
    let filter_name = syn::Ident::new(&format!("{struct_name}Filter"), struct_name.span());

    let fields = match &derive_input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(named) => &named.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let proxy_fields = fields.iter().map(|field| {
        let name = &field.ident;
        let ty = &field.ty;
        quote! {
            pub #name: ormolu_interfaces::field::Col<#ty>
        }
    });

    let proxy_inits = fields.iter().map(|field| {
        let name = &field.ident;
        let field_name = name.as_ref().unwrap().to_string();
        quote! {
            #name: Col::new(#field_name, state.clone())
        }
    });

    quote! {
        pub struct #filter_name {
            #( #proxy_fields, )*
        }

        impl #filter_name {
            pub fn new(state: std::rc::Rc<std::cell::RefCell<ormolu_interfaces::field::filter::FilterState>>) -> Self {
                Self {
                    #( #proxy_inits, )*
                }
            }
        }

        impl ormolu_interfaces::Filterable for #struct_name {
            type Proxy = #filter_name;

            fn to_field_filter(state: std::rc::Rc<std::cell::RefCell<ormolu_interfaces::field::filter::FilterState>>) -> Self::Proxy {
                #filter_name::new(state)
            }
        }
    }.into()
}

pub fn selectable(derive_input: &DeriveInput) -> TokenStream {
    let struct_name = &derive_input.ident;
    let select_name = syn::Ident::new(&format!("{struct_name}Select"), struct_name.span());

    let fields = match &derive_input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(named) => &named.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let proxy_fields = fields.iter().map(|field| {
        let name = &field.ident;
        quote! {
            pub #name: bool
        }
    });

    quote! {
        #[derive(Default)]
        pub struct #select_name {
            #( #proxy_fields, )*
        }

        impl ormolu_interfaces::Selectable for #struct_name {
            type Select = #select_name;

            /// Returns a struct for picking specific fields to select.
            ///
            /// TODO: select doc comment example
            fn select() -> Self::Select {
                #select_name::default()
            }
        }
    }
    .into()
}
