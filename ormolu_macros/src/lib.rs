use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::*;

struct TableMetadata {
    /// name of the struct
    pub struct_name: Ident,
    pub name: Option<String>,
    pub schema: Option<String>,
    // HashMap?
    pub fields: Vec<ColumnMetadata>,
}

impl TableMetadata {
    pub fn new(struct_name: Ident) -> Self {
        Self {
            struct_name,
            fields: Vec::new(),
            name: None,
            schema: None,
        }
    }
}

struct ColumnMetadata {
    /// Name of the current column
    pub name: Ident,
    pub ty: Type,
    /// `true` if the current column is a primary key
    pub primary_key: bool,
    /// `true` if the current column is unique
    pub unique: bool,
    /// `Some("Customer", "customer_id")` (type, field_name)
    pub references: Option<(String, String)>,
}

impl ColumnMetadata {
    pub fn new(name: Ident, ty: Type) -> Self {
        Self {
            name,
            ty,
            primary_key: false,
            unique: false,
            references: None,
        }
    }
}

fn convert_to_token_stream(table_metadata: TableMetadata) -> TokenStream {
    let mut tk2 = TokenStream2::new();

    let struct_name = table_metadata.struct_name;

    let table_name = table_metadata.name.expect("expected table_name");
    let has_table_name_impl = quote! {
        impl ormolu_interfaces::traits::HasTableName for #struct_name {
            fn table_name() -> &'static str {
                #table_name
            }
        }
    };
    has_table_name_impl.to_tokens(&mut tk2);

    if let Some(schema_name) = table_metadata.schema {
        let has_schema_name_impl = quote! {
            impl ormolu_interfaces::traits::HasSchemaName for #struct_name {
                fn schema_name() -> &'static str {
                    #schema_name
                }
            }
        };
        has_schema_name_impl.to_tokens(&mut tk2);
    }

    let field_names: Vec<String> = table_metadata
        .fields
        .iter()
        .map(|f| f.name.to_string())
        .collect();

    // Convert Vec<String> to Vec<TokenStream>
    let quoted_strings: Vec<proc_macro2::TokenStream> = field_names
        .into_iter()
        .map(|s| {
            let lit = syn::LitStr::new(&s, proc_macro2::Span::call_site());
            quote! { #lit.to_string() }
        })
        .collect();

    let fields_getter = quote! {
        impl ormolu_interfaces::traits::HasFields  for #struct_name {
            /// Returns the rust struct fields
            fn fields() -> Vec<String> {
                vec![#(#quoted_strings),*]
            }
        }
    };
    fields_getter.to_tokens(&mut tk2);

    for field in table_metadata.fields {
        let field_name_lit = field.name.clone().to_string();

        let field_name = field.name;
        let field_type = field.ty;

        if field.primary_key {
            let primary_key_impl = quote! {
                impl ormolu_interfaces::traits::HasPrimaryKey<#field_type> for #struct_name {
                    fn primary_key(&self) -> &#field_type {
                        &self.#field_name
                    }

                    fn primary_key_field_name() -> &'static str {
                        #field_name_lit
                    }
                }
            };

            primary_key_impl.to_tokens(&mut tk2);
        }
        if field.unique {
            let method_name =
                Ident::new(format!("find_by_{field_name}").as_str(), Span::call_site());

            let unique_field_find_method = quote! {
                impl #struct_name {
                    fn #method_name(#field_name: #field_type) -> bool {
                        true
                    }
                }
            };

            unique_field_find_method.to_tokens(&mut tk2);
        }
    }

    let is_gilded = quote! {
        impl ormolu_interfaces::traits::Gilded for #struct_name {}
    };
    is_gilded.to_tokens(&mut tk2);

    let code = tk2.to_string();
    std::fs::create_dir_all("/tmp/ormolu/").expect("could not create /tmp/ormolu");
    let filename = format!("/tmp/ormolu/{}.rs", "ormolu");
    std::fs::write(filename, code).expect("could not write temp expanded macro");

    tk2.into()
}

#[proc_macro_derive(ToProxy)]
pub fn derive_to_proxy(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let proxy_name = syn::Ident::new(&format!("{struct_name}Filter"), struct_name.span());

    let fields = match &input.data {
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
            pub #name: ormolu_interfaces::field::Field<#ty>
        }
    });

    let proxy_inits = fields.iter().map(|field| {
        let name = &field.ident;
        let field_name = name.as_ref().unwrap().to_string();
        quote! {
            #name: Field::new(#field_name, state.clone())
        }
    });

    let output = quote! {
        /// cool ass struct
        pub struct #proxy_name {
            #( #proxy_fields, )*
        }

        impl #proxy_name {
            pub fn new(state: std::rc::Rc<std::cell::RefCell<QueryState>>) -> Self {
                Self {
                    #( #proxy_inits, )*
                }
            }
        }


        impl ormolu_interfaces::traits::ToProxy for #struct_name {
            type Proxy = #proxy_name;

            fn to_proxy(state: std::rc::Rc<std::cell::RefCell<ormolu_interfaces::sql_command::query::query_builder::QueryState>>) -> Self::Proxy {
                #proxy_name::new(state)
            }
        }
    };

    output.into()
}

// These are the attributes adorned the top of a struct
//
// ```no_run
// #[gild(table = "product", schema = "public")]
// pub struct Product {
//     ...
// }
// ```
// fn parse_header_attributes(input: &DeriveInput) -> TokenStream {
//     let mut table_name = TokenStream2::new();
//     let mut schema_name = TokenStream2::new();

//     for attr in input.attrs.iter() {
//         if !attr.path.is_ident("gild") {
//             // probably something else
//             continue;
//             // return Error::new_spanned(
//             //     attr,
//             //     format!(
//             //         "unknown attribute: {}",
//             //         attr.path
//             //             .get_ident()
//             //             .expect("failed to get_ident for name_value.path")
//             //     ),
//             // )
//             // .to_compile_error()
//             // .into();
//         }

//         let Ok(Meta::List(meta_list)) = attr.parse_meta() else {
//             continue;
//         };

//         for nested_meta in meta_list.nested.iter() {
//             let syn::NestedMeta::Meta(Meta::NameValue(name_value)) = nested_meta else {
//                 return Error::new_spanned(nested_meta, "gild attribute missing value")
//                     .to_compile_error()
//                     .into();
//             };

//             let Lit::Str(lit_str) = &name_value.lit else {
//                 return Error::new_spanned(nested_meta, "gild attribute missing value")
//                     .to_compile_error()
//                     .into();
//                 // continue;
//             };

//             let str_lit_value = lit_str.value();

//             let Some(id) = name_value.path.get_ident() else {
//                 return Error::new_spanned(
//                     name_value,
//                     format!(
//                         "missing struct attribute for parameter: {}",
//                         name_value
//                             .path
//                             .get_ident()
//                             .expect("failed to get_ident for name_value.path")
//                     ),
//                 )
//                 .to_compile_error()
//                 .into();
//             };

//             match id.to_string().as_str() {
//                 "table" => {
//                     table_name = quote! {
//                         pub fn table_name() -> &'static str {
//                             #str_lit_value
//                         }
//                     };
//                 }
//                 "schema" => {
//                     schema_name = quote! {
//                         pub fn schema_name() -> &'static str {
//                             #str_lit_value
//                         }
//                     };
//                 }
//                 _ => {
//                     return Error::new_spanned(
//                         name_value,
//                         format!(
//                             "unknown struct attribute parameter: {}",
//                             name_value
//                                 .path
//                                 .get_ident()
//                                 .expect("failed to get_ident for name_value.path")
//                         ),
//                     )
//                     .to_compile_error()
//                     .into();
//                 }
//             }
//         }
//     }

//     if table_name.is_empty() {
//         return Error::new_spanned(input, "requires '#[gild(table = \"...\")]' to be set")
//             .to_compile_error()
//             .into();
//     }

//     let struct_name = &input.ident;

//     TokenStream::from(quote! {
//         impl #struct_name {
//             #table_name
//             #schema_name
//         }
//     })
// }

fn parse_header_attributes2(
    input: &DeriveInput,
    mut table: TableMetadata,
) -> Result<TableMetadata> {
    for attr in input.attrs.iter() {
        if !attr.path.is_ident("gild") {
            // ignore
            continue;
        }

        let Ok(Meta::List(meta_list)) = attr.parse_meta() else {
            continue;
        };

        for nested_meta in meta_list.nested.iter() {
            let syn::NestedMeta::Meta(Meta::NameValue(name_value)) = nested_meta else {
                return Err(Error::new_spanned(
                    nested_meta,
                    "gild attribute missing value",
                ));
            };

            let Lit::Str(lit_str) = &name_value.lit else {
                return Err(Error::new_spanned(
                    nested_meta,
                    "gild attribute missing value",
                ));
            };

            let str_lit_value = lit_str.value();

            let Some(id) = name_value.path.get_ident() else {
                return Err(Error::new_spanned(
                    name_value,
                    format!(
                        "missing struct attribute for parameter: {}",
                        name_value
                            .path
                            .get_ident()
                            .expect("failed to get_ident for name_value.path")
                    ),
                ));
            };

            match id.to_string().as_str() {
                "table" => table.name = Some(str_lit_value),
                "schema" => table.schema = Some(str_lit_value),
                _ => {
                    return Err(Error::new_spanned(
                        name_value,
                        format!(
                            "unknown struct attribute parameter: {}",
                            name_value
                                .path
                                .get_ident()
                                .expect("failed to get_ident for name_value.path")
                        ),
                    ))
                }
            }
        }
    }

    if table.name.is_none() {
        return Err(Error::new_spanned(
            input,
            "requires '#[gild(table = \"...\")]' to be set",
        ));
    }

    // let struct_name = &input.ident;

    // TokenStream::from(quote! {
    //     impl #struct_name {
    //         #table_name
    //         #schema_name
    //     }
    // })
    Ok(table)
}

// These are the attributes within the struct body
//
// ```no_run
// #[gild(table = "product", schema = "public")]
// pub struct Product {
//     #[gild(primary_key)] // <- these
//     id: i32
// }
// ```
// fn parse_body_attributes(input: &DeriveInput) -> TokenStream {
//     let mut tk2 = TokenStream2::new();

//     let syn::Data::Struct(data_struct) = &input.data else {
//         unimplemented!("only structs supported currently")
//     };

//     let Fields::Named(fields) = &data_struct.fields else {
//         unimplemented!("currently requires named fields")
//     };

//     let struct_name = &input.ident;

//     let mut field_names: Vec<String> = Vec::new();

//     for field in &fields.named {
//         let field_name = field.ident.clone().expect("expected identifier");
//         field_names.push(field_name.to_string());

//         for attr in &field.attrs {
//             if !attr.path.is_ident("gild") {
//                 continue;
//             }

//             let Ok(Meta::List(meta_list)) = attr.parse_meta() else {
//                 continue;
//             };

//             for nested_meta in meta_list.nested.iter() {
//                 let syn::NestedMeta::Meta(Meta::Path(path)) = nested_meta else {
//                     continue;
//                 };

//                 if path.is_ident("primary_key") {
//                     let field_type = field.ty.clone();
//                     let field_name = field.ident.clone().expect("expected ident");
//                     let field_name_lit = field_name.to_string();

//                     let primary_key_impl = quote! {
//                         impl PrimaryKeyConstraint<#field_type> for #struct_name {
//                             fn primary_key(&self) -> &#field_type {
//                                 &self.#field_name
//                             }

//                             fn primary_key_field_name() -> &'static str {
//                                 #field_name_lit
//                             }
//                         }
//                     };

//                     primary_key_impl.to_tokens(&mut tk2);
//                     // tk2.append();
//                 } else if path.is_ident("unique") {
//                     let method_name =
//                         Ident::new(format!("find_by_{field_name}").as_str(), path.span());
//                     let field_ty = &field.ty;

//                     let unique_field_find_method = quote! {
//                         impl #struct_name {
//                             fn #method_name(#field_name: #field_ty) -> bool {
//                                 true
//                             }
//                         }
//                     };

//                     unique_field_find_method.to_tokens(&mut tk2);
//                 } else {
//                     let unknown_thing = path.get_ident().expect("get ident").to_string();

//                     return Error::new_spanned(
//                         path,
//                         format!("unknown gild attribute parameter: {unknown_thing}"),
//                     )
//                     .to_compile_error()
//                     .into();
//                 }
//             }
//         }
//     }

//     // Convert Vec<String> to Vec<TokenStream>
//     let quoted_strings: Vec<proc_macro2::TokenStream> = field_names
//         .into_iter()
//         .map(|s| {
//             let lit = syn::LitStr::new(&s, proc_macro2::Span::call_site());
//             quote! { #lit.to_string() }
//         })
//         .collect();

//     let fields_getter = quote! {
//         impl #struct_name {
//             /// Returns the rust struct fields
//             fn fields() -> Vec<String> {
//                 vec![#(#quoted_strings),*]
//             }
//         }
//     };

//     fields_getter.to_tokens(&mut tk2);

//     tk2.into()
// }

fn parse_body_attributes2(input: &DeriveInput, mut table: TableMetadata) -> Result<TableMetadata> {
    let syn::Data::Struct(data_struct) = &input.data else {
        unimplemented!("only structs supported currently")
    };

    let Fields::Named(fields) = &data_struct.fields else {
        unimplemented!("currently requires named fields")
    };

    for field in &fields.named {
        let field_name = field.ident.clone().expect("expected identifier");
        let mut col = ColumnMetadata::new(field_name, field.ty.clone());

        for attr in &field.attrs {
            if !attr.path.is_ident("gild") {
                continue;
            }

            let Ok(Meta::List(meta_list)) = attr.parse_meta() else {
                continue;
            };

            for nested_meta in meta_list.nested.iter() {
                let syn::NestedMeta::Meta(Meta::Path(path)) = nested_meta else {
                    continue;
                };

                if path.is_ident("primary_key") {
                    col.primary_key = true;
                    col.unique = true;
                } else if path.is_ident("unique") {
                    col.unique = true;
                } else {
                    let unknown_thing = path.get_ident().expect("get ident").to_string();

                    return Err(Error::new_spanned(
                        path,
                        format!("unknown gild attribute parameter: {unknown_thing}"),
                    ));
                }
            }
        }

        table.fields.push(col);
    }

    Ok(table)
}

#[proc_macro_derive(Gilded, attributes(gild))]
pub fn gilded_derive(input: TokenStream) -> TokenStream {
    let proxied: TokenStream2 = derive_to_proxy(input.clone()).into();

    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident.clone();

    let table_metadata = TableMetadata::new(struct_name);

    let table_metadata = match parse_header_attributes2(&input, table_metadata) {
        Ok(table_metadata) => table_metadata,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let table_metadata = match parse_body_attributes2(&input, table_metadata) {
        Ok(table_metadata) => table_metadata,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    // let header_attributes: TokenStream2 = parse_header_attributes(&input).into();
    // let body_attributes: TokenStream2 = parse_body_attributes(&input).into();

    let gilded: TokenStream2 = convert_to_token_stream(table_metadata).into();

    let q = quote! {
        // #imports
        #proxied
        #gilded
    };

    q.into()
}
