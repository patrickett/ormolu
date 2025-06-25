use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

pub fn expand_derive_database_object(derive_input: &DeriveInput) -> syn::Result<TokenStream> {
    // can we not be a String?
    let mut object_name = String::new();
    let mut schema_name = String::new();
    let struct_name = &derive_input.ident;

    for attr in derive_input.attrs.iter() {
        let Some(ident) = attr.path().get_ident() else {
            continue;
        };

        let Meta::NameValue(MetaNameValue { value, .. }) = &attr.meta else {
            continue;
        };

        let Expr::Lit(ExprLit { lit, .. }) = value else {
            continue;
        };

        let Lit::Str(litstr) = lit else {
            continue;
        };

        match ident.to_string().as_str() {
            "name" => {
                let value = litstr.value();
                let parts: Vec<&str> = value.split(".").collect();

                match parts.as_slice() {
                    [first, second] => {
                        schema_name = first.to_string();
                        object_name = second.to_string();
                    }
                    _ => {
                        // TODO: add hint here
                        return Err(Error::new_spanned(
                            derive_input,
                            "requires '#[name = \"...\"]' attribute to specify the correct database object",
                        ));
                    }
                };
            }
            "object" => object_name = litstr.value(),
            "schema" => schema_name = litstr.value(),
            "get_pool" => todo!(),
            aname => {
                return Err(Error::new_spanned(
                    derive_input,
                    format!("unknown attribute: '{aname}'"),
                ))
            }
        }
    }

    if object_name.is_empty() {
        return Err(Error::new_spanned(
            derive_input,
            "requires '#[object = \"...\"]' attribute to specify the correct database object",
        ));
    }

    if schema_name.is_empty() {
        return Err(Error::new_spanned(
            derive_input,
            "requires '#[schema = \"...\"]' attribute to specify the correct database schema.object",
        ));
    }

    let qualified_name = format!("{schema_name}.{object_name}");

    Ok(quote! {
        // TODO: HasName -> HasObjectName
        // TODO: name() -> object_name()
        impl ormolu_interfaces::HasObjectName for #struct_name {
            /// The name of the table
            fn object_name() -> &'static str {
                #object_name
            }
        }

        // TODO: HasSchemaName -> HasSchema
        impl ormolu_interfaces::HasSchemaName for #struct_name {
            fn schema_name() -> &'static str {
                #schema_name
            }
        }

        impl ormolu_interfaces::HasQualifiedName for #struct_name {
            fn qualified_name() -> &'static str {
                #qualified_name
            }
        }

        impl ormolu_interfaces::GetConnectionPool for #struct_name {}
        impl ormolu_interfaces::DatabaseObject for #struct_name {}
    })
}
