use crate::utils::*;
use ormolu_interfaces::{ToPlural, ToSingular};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, TokenStreamExt};
use syn::parse::Parse;
use syn::{parse::ParseStream, *};

/// This is just a helper struct for managing the proc macro state.
///
/// We parse and iterate over the entire struct then will convert that into
/// the necessary traits
pub struct TableMetadata<'de> {
    /// name of the struct (Pascal cased table name)
    pub struct_name: &'de Ident,
    // HashMap?
    pub fields: Vec<ColumnMetadata>,
}

impl<'de> TableMetadata<'de> {
    pub fn new(struct_name: &'de Ident) -> Self {
        Self {
            struct_name,
            fields: Vec::new(),
        }
    }

    #[inline]
    fn expanded_reflection(&self) -> TokenStream2 {
        let exceptions: Vec<_> = self
            .fields
            .iter()
            .filter_map(|col| {
                col.attributes.column_name.as_ref().map(|db_name| {
                    let key = col.name.to_string();
                    quote::quote! {
                        #key => #db_name
                    }
                })
            })
            .collect();

        let struct_name = &self.struct_name;

        let screaming_struct_name = struct_name.to_string().to_uppercase();
        let exceptions_name = Ident::new(
            format!("{screaming_struct_name}_EXCEPTIONS").as_str(),
            Span::call_site(),
        );

        let field_names: Vec<String> = self.fields.iter().map(|f| f.name.to_string()).collect();
        let field_strs: Vec<proc_macro2::TokenStream> =
            field_names.iter().map(|name| quote! { #name }).collect();
        quote! {
            static #exceptions_name: phf::Map<&'static str, &'static str> = phf::phf_map! {
                #(#exceptions),*
            };

            impl ormolu_interfaces::HasFields for #struct_name {
                /// Returns list of field names of the struct
                fn field_map() -> &'static phf::Map<&'static str, &'static str> {
                    &#exceptions_name
                }

                fn database_columns() -> &'static [&'static str] {
                    &[#(#field_strs),*]
                }
            }
        }
    }

    #[inline]
    fn expanded_primary_key(&self, field: &ColumnMetadata) -> TokenStream2 {
        if !field.constraints.primary_key {
            return TokenStream2::new();
        }

        // TODO: remove this clone?
        let field_name = &field.name.clone();
        let field_type = &field.ty;
        let struct_name = &self.struct_name;

        let method_name = Ident::new(format!("get_by_{field_name}").as_str(), Span::call_site());

        quote! {
            impl ormolu_interfaces::HasPrimaryKey<#field_type> for #struct_name {
                fn primary_key(&self) -> &#field_type {
                    &self.#field_name
                }

                // fn primary_key_field_name() -> &'static str {
                //     stringify!(#field_name)
                // }

                fn get_by_primary_key(key: &#field_type) -> impl Future<Output = Result<Option<Self>, ormolu_interfaces::OrmoluError>> {
                    Self::#method_name(*key)
                }
            }
        }
    }

    #[inline]
    fn expanded_unique(&self, field: &ColumnMetadata) -> TokenStream2 {
        if !field.constraints.unique {
            return TokenStream2::new();
        }

        // TODO: remove this clone?
        let field_name = &field.name.clone();
        let field_type = replace_string_with_into_string(unwrap_unique_or_self(
            unwrap_option_or_self(&field.ty),
        ));

        let struct_name = &self.struct_name;
        let method_name = Ident::new(format!("get_by_{field_name}").as_str(), Span::call_site());
        let sql_value_string = generate_field_to_string_expr(field_name, &field_type);
        quote! {
            impl #struct_name {
                // TODO: make type a ref?
                fn #method_name(#field_name: #field_type) -> impl Future<Output = Result<Option<Self>, ormolu_interfaces::OrmoluError>> {
                    // TODO: this feels like it is slightly poorly designed
                    async move {
                        let mut q: ormolu_interfaces::sql_command::query::QueryState<Self> = ormolu_interfaces::sql_command::query::QueryState::new_select();
                        let s: String = #sql_value_string;
                        let db_col_name = Self::get_db_column_name(stringify!(#field_name));
                        let where_cond = ormolu_interfaces::sql_command::query::where_cond::Where::eq(db_col_name, s);
                        q.where_conditions.push(where_cond);

                        let pool = Self::get_connection_pool().await;

                        Ok(
                            sqlx::query_as::<_, Self>(q.to_string().as_str())
                                .fetch_optional(&pool)
                                .await?,
                        )
                    }
                }
            }
        }
    }

    #[inline]
    fn expanded_impls(&self) -> TokenStream2 {
        let struct_name = &self.struct_name;
        quote! {
            impl ormolu_interfaces::Table for #struct_name {}
        }
    }

    #[inline]
    fn expanded_references(&self, field: &ColumnMetadata) -> TokenStream2 {
        let Some((ident, _ord)) = &field.constraints.foreign_key else {
            return TokenStream2::new();
        };

        let other_model = ident;
        let struct_name = &self.struct_name;
        let singular = Ident::new(
            other_model
                .to_string()
                .to_lowercase()
                .to_singular()
                .as_str(),
            Span::call_site(),
        );
        let many = Ident::new(
            struct_name.to_string().to_lowercase().to_plural().as_str(),
            Span::call_site(),
        );

        // references exist on the many to one

        quote! {
            use ormolu_interfaces::sql_command::query::*;
            // many
            impl #struct_name {
                fn #singular(&self) -> QuerySet<#other_model> {
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
                fn #many(&self) -> QuerySet<#struct_name> {
                    let query_state: ormolu_interfaces::sql_command::query::QueryState<#struct_name> = QueryState::new_select();
                    let query_set = ormolu_interfaces::sql_command::query::QuerySet::new(query_state);
                    // query_set.filter(|m| m.customer_id == self.id)
                    query_set
                }
            }
        }
    }

    pub fn to_token_stream(&self) -> TokenStream2 {
        let mut stream = TokenStream2::new();

        stream.append_all([self.expanded_reflection(), self.expanded_impls()]);

        for field in &self.fields {
            stream.append_all([
                self.expanded_primary_key(field),
                self.expanded_unique(field),
                self.expanded_references(field),
            ]);
        }

        // let code = tk2.to_string();
        // std::fs::create_dir_all("/tmp/ormolu/").expect("could not create /tmp/ormolu");
        // let filename = format!("/tmp/ormolu/{}.rs", "ormolu");
        // std::fs::write(filename, code).expect("could not write temp expanded macro");

        stream
    }
}

#[derive(Default)]
pub struct ColumnConstraints {
    // ordinal: usize,
    primary_key: bool,
    foreign_key: Option<(Ident, usize)>,
    unique: bool,
}

pub struct ColumnMetadata {
    /// Name of the rust struct field
    pub name: Ident,
    pub ty: Type,
    /// These come from attribute macros on the field of the struct
    pub attributes: FieldAttributes,

    pub constraints: ColumnConstraints,
}

impl ColumnMetadata {
    pub fn new(name: Ident, ty: Type) -> Self {
        let attributes = FieldAttributes::default();
        let constraints = ColumnConstraints::default();
        Self {
            name,
            ty,
            attributes,
            constraints,
        }
    }
}

pub fn expand_derive_table(derive_input: &DeriveInput) -> Result<TableMetadata> {
    let struct_name = &derive_input.ident;
    let mut table = TableMetadata::new(struct_name);

    let syn::Data::Struct(data_struct) = &derive_input.data else {
        unimplemented!("only structs supported currently")
    };

    let Fields::Named(fields) = &data_struct.fields else {
        unimplemented!("currently requires named fields")
    };

    for field in &fields.named {
        let field_name = field.ident.clone().expect("expected identifier");
        let mut col = ColumnMetadata::new(field_name, field.ty.clone());

        let ct = parse_custom_type(&field.ty);

        let mut primary_key = false;
        let mut unique = false;
        let mut foreign_key: Option<(Ident, usize)> = None;

        match ct {
            CustomType::PrimaryKey(_, _) => {
                primary_key = true;
                unique = true;
            }
            CustomType::Unique(_) => {
                unique = true;
            }
            CustomType::ForeignKey(entity, ordinal, _type) => {
                // unique = true;
                foreign_key = Some((entity.get_ident().expect("ident").to_owned(), ordinal))
            }
            CustomType::Other(_) => {}
        }

        col.constraints = ColumnConstraints {
            primary_key,
            unique: primary_key || unique,
            foreign_key,
        };

        for attr in &field.attrs {
            if attr.path().is_ident("gild") {
                if let Ok(attributes) = attr.parse_args::<FieldAttributes>() {
                    col.attributes = attributes;
                };
            }
        }

        table.fields.push(col);
    }

    Ok(table)
}

// Constraints are defined by the gild attributes
#[derive(Default)]
pub struct FieldAttributes {
    /// Actual database column name
    pub column_name: Option<String>,
}

impl Parse for FieldAttributes {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut constraints = FieldAttributes::default();

        while !input.is_empty() {
            let lookahead = input.lookahead1();

            if lookahead.peek(Ident) {
                let ident: Ident = input.parse()?;

                match ident.to_string().as_str() {
                    // "primary_key" => {
                    //     constraints.unique = true;
                    //     constraints.primary_key = true;
                    // }
                    // "unique" => {
                    //     constraints.unique = true;
                    // }
                    "column" => {
                        input.parse::<Token![=]>()?;
                        let expr: Expr = input.parse()?;
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        }) = expr
                        {
                            constraints.column_name = Some(s.value())
                        } else {
                            return Err(input.error("Expected string literal for column"));
                        }
                    }
                    // "references" => {
                    //     input.parse::<Token![=]>()?;
                    //     let expr: Expr = input.parse()?;
                    //     if let Expr::Tuple(ExprTuple { elems, .. }) = expr {
                    //         let mut elems = elems.into_iter();

                    //         let model = match elems.next() {
                    //             Some(Expr::Path(p)) => p.path,
                    //             _ => {
                    //                 return Err(
                    //                     input.error("Expected type path as first tuple element")
                    //                 )
                    //             }
                    //         };
                    //         let field = match elems.next() {
                    //             Some(Expr::Lit(ExprLit {
                    //                 lit: Lit::Str(s), ..
                    //             })) => s.value(),
                    //             _ => {
                    //                 return Err(input
                    //                     .error("Expected string literal as second tuple element"))
                    //             }
                    //         };

                    //         constraints.references = Some(References { model, field })
                    //     } else {
                    //         return Err(input.error("Expected tuple for references"));
                    //     }
                    // }
                    _ => return Err(input.error("Unknown gild attribute")),
                }

                // Optionally parse a trailing comma
                let _ = input.parse::<Token![,]>();
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(constraints)
    }
}
