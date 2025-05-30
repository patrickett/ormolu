use ormolu_interfaces::{ToPlural, ToSingular};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    *,
};

struct TableMetadata {
    /// name of the struct (Pascal cased table name)
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
    /// Actual database column name
    pub column_name: Option<String>,
    pub ty: Type,
    /// `true` if the current column is a primary key
    pub primary_key: bool,
    /// `true` if the current column is unique
    pub unique: bool,
    /// `Some("Customer", "customer_id")` (type, field_name)
    pub references: Option<References>,
}

impl ColumnMetadata {
    pub fn new(name: Ident, ty: Type) -> Self {
        Self {
            name,
            ty,
            column_name: None,
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

    let field_strs: Vec<proc_macro2::TokenStream> =
        field_names.iter().map(|name| quote! { #name }).collect();

    let fields_getter = quote! {
        impl ormolu_interfaces::traits::HasFields for #struct_name {
            /// Returns list of field names of the struct
            fn fields() -> &'static [&'static str] {
                &[#(#field_strs),*]
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
        if let Some(reference) = field.references {
            let other_model = reference.model;

            let singular = Ident::new(
                other_model
                    .clone()
                    .to_string()
                    .to_lowercase()
                    .to_singular()
                    .as_str(),
                Span::call_site(),
            );
            let many = Ident::new(
                struct_name
                    .clone()
                    .to_string()
                    .to_lowercase()
                    .to_plural()
                    .as_str(),
                Span::call_site(),
            );

            // references exist on the many to one

            let relationship_queries = quote! {
                // many
                impl #struct_name {
                    fn #singular(&self) -> ormolu_interfaces::sql_command::query::QuerySet<#other_model> {
                        let query_state: ormolu_interfaces::sql_command::query::QueryState<#other_model> = QueryState::new_select();
                        let query_set = ormolu_interfaces::sql_command::query::QuerySet::new(query_state);
                        // TODO: actually filter based on the field
                        // issue here is if we have a column field renamed
                        // query_set.filter(|m| m.customer_id == self.id)
                        query_set
                    }
                }


                // one
                impl #other_model {
                    fn #many(&self) -> ormolu_interfaces::sql_command::query::QuerySet<#struct_name> {
                        let query_state: ormolu_interfaces::sql_command::query::QueryState<#struct_name> = QueryState::new_select();
                        let query_set = ormolu_interfaces::sql_command::query::QuerySet::new(query_state);
                        // query_set.filter(|m| m.customer_id == self.id)
                        query_set
                    }
                }
            };

            relationship_queries.to_tokens(&mut tk2);
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

#[proc_macro_derive(Selectable)]
pub fn derive_selectable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let select_name = syn::Ident::new(&format!("{struct_name}Select"), struct_name.span());

    let fields = match &input.data {
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

    let output = quote! {
        #[derive(Default)]
        pub struct #select_name {
            #( #proxy_fields, )*
        }

        impl ormolu_interfaces::traits::Selectable for #struct_name {
            type Select = #select_name;

            /// Returns a struct for picking specific fields to select.
            ///
            /// TODO: select doc comment example
            fn select() -> Self::Select {
                #select_name::default()
            }
        }
    };

    output.into()
}

#[proc_macro_derive(Filterable)]
pub fn derive_to_field_filter(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let filter_name = syn::Ident::new(&format!("{struct_name}Filter"), struct_name.span());

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

        impl ormolu_interfaces::traits::Filterable for #struct_name {
            type Proxy = #filter_name;

            fn to_field_filter(state: std::rc::Rc<std::cell::RefCell<ormolu_interfaces::field::filter::FilterState>>) -> Self::Proxy {
                #filter_name::new(state)
            }
        }
    };

    output.into()
}

/// These are the attributes adorned the top of a struct
///
/// ```no_run
/// #[gild(table = "product", schema = "public")]
/// pub struct Product {
///     ...
/// }
/// ```
fn parse_header_attributes(input: &DeriveInput, mut table: TableMetadata) -> Result<TableMetadata> {
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
                // TODO: if no connection getter specified use a default
                // lazy static once cell thing. such that per model if you
                // so choose you can easily pass a different method for its getter
                "connection" => todo!(),
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

    Ok(table)
}

#[derive(Clone)]
struct References {
    model: Ident,
    // _comma: Token![,],
    field: LitStr,
}

enum OrmArg {
    References(References),
    // OnDelete(LitStr),
}

struct OrmArgs {
    args: Vec<OrmArg>,
}

impl Parse for OrmArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let key: Ident = input.parse()?; // `references`
        let _eq: Token![=] = input.parse()?; // `=`
        let content;
        syn::parenthesized!(content in input); // Parse inside `(...)`

        let model: Ident = content.parse()?; // `Order`
        let _comma: Token![,] = content.parse()?; // `,`
        let field: LitStr = content.parse()?; // `"order_id"`

        let refrences = References {
            model,
            // _comma: comma,
            field,
        };

        Ok(OrmArgs {
            args: vec![OrmArg::References(refrences)],
        })
    }
}

/// These are the attributes within the struct body
///
/// ```no_run
/// #[gild(table = "product", schema = "public")]
/// pub struct Product {
///     #[gild(primary_key)] // <- these
///     id: i32
/// }
/// ```
fn parse_body_attributes(input: &DeriveInput, mut table: TableMetadata) -> Result<TableMetadata> {
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

            if let Ok(meta) = attr.parse_meta() {
                match meta {
                    Meta::List(meta_list) => {
                        for nested_meta in meta_list.nested.iter() {
                            match nested_meta {
                                NestedMeta::Meta(meta) => match meta {
                                    Meta::List(list) => {
                                        if list.path.is_ident("references") {
                                            panic!("got here ref")
                                        } else {
                                            panic!("other 1")
                                        }
                                    }
                                    Meta::NameValue(meta_name_value) => {
                                        if meta_name_value.path.is_ident("references") {
                                            panic!("got here ref")
                                        } else if meta_name_value.path.is_ident("column") {
                                            match meta_name_value.lit.clone() {
                                                // NOTE: this is how we rename columns.
                                                // ex. 'order_id' -> 'id'
                                                Lit::Str(s) => {
                                                    col.column_name = Some(s.value());
                                                }
                                                _ => panic!("other"),
                                            }
                                        } else {
                                            match meta_name_value.lit.clone() {
                                                Lit::Str(s) => {
                                                    panic!("b: {}", s.value())
                                                }
                                                _ => panic!("other"),
                                            }
                                        }
                                    }
                                    Meta::Path(path) => {
                                        if path.is_ident("primary_key") {
                                            col.primary_key = true;
                                            col.unique = true;
                                        } else if path.is_ident("unique") {
                                            col.unique = true;
                                        } else {
                                            panic!("got here 123")
                                        }
                                    }
                                },
                                NestedMeta::Lit(lit) => match lit {
                                    Lit::Str(s) => {
                                        panic!("lit str lit")
                                    }
                                    _ => panic!("sd"),
                                },
                            };
                        }
                    }
                    Meta::Path(path) => panic!("meta path"),
                    Meta::NameValue(name_value) => panic!(" name value"),
                };
            }

            if let Ok(args) = attr.parse_args::<OrmArgs>() {
                // TODO: this is bad but will work for now
                if let Some(OrmArg::References(refer)) = args.args.first() {
                    col.references = Some(refer.to_owned());
                }
            }

            // if let Ok(args) = attr.parse_args() {}
        }

        table.fields.push(col);
    }

    Ok(table)
}

fn assert_gilded_impl_from_row(struct_name: Ident) -> TokenStream {
    let ident = struct_name;
    quote! {
        // const _: fn() = || {
        //     fn assert_from_row<T>()
        //     where
        //         T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>,
        //     {}
        //     assert_from_row::<#ident>();
        // };
    }
    .into()
}

/// Requires [`sqlx::FromRow`] to be derived on the same struct.
#[proc_macro_derive(Gilded, attributes(gild))]
pub fn gilded_derive(input: TokenStream) -> TokenStream {
    // PERF: remove the input.clone() parse all in one go
    let field_filter: TokenStream2 = derive_to_field_filter(input.clone()).into();
    let selectable: TokenStream2 = derive_selectable(input.clone()).into();
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident.clone();
    let sqlx_stuff: TokenStream2 = assert_gilded_impl_from_row(struct_name.clone()).into();

    let table_metadata = TableMetadata::new(struct_name);

    let table_metadata = match parse_header_attributes(&input, table_metadata) {
        Ok(table_metadata) => table_metadata,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let table_metadata = match parse_body_attributes(&input, table_metadata) {
        Ok(table_metadata) => table_metadata,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let gilded: TokenStream2 = convert_to_token_stream(table_metadata).into();

    let q = quote! {
        #field_filter
        #sqlx_stuff
        #selectable
        #gilded
    };

    q.into()
}
