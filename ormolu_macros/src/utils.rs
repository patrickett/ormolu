use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

/// If the type is exactly `String`, return `Into<String>`. Otherwise, clone and return the original.
pub fn replace_string_with_into_string(ty: &Type) -> Type {
    if let Type::Path(TypePath { qself: None, path }) = ty {
        if let Some(last) = path.segments.last() {
            if last.ident == "String" && path.segments.len() == 1 {
                // It's exactly `String`
                return parse_quote! { impl Into<String> };
            }
        }
    }

    ty.clone()
}

/// If the type is `Option<T>`, return `T`. Otherwise, return the original type.
pub fn unwrap_option_or_self(ty: &Type) -> &Type {
    if let Type::Path(type_path) = ty {
        if type_path.qself.is_none() {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Option" {
                    if let PathArguments::AngleBracketed(ref args) = segment.arguments {
                        if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                            return inner_ty;
                        }
                    }
                }
            }
        }
    }

    ty
}

/// If the type is `Option<T>`, return `T`. Otherwise, return the original type.
pub fn unwrap_unique_or_self(ty: &Type) -> &Type {
    if let Type::Path(type_path) = ty {
        if type_path.qself.is_none() {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Unique" {
                    if let PathArguments::AngleBracketed(ref args) = segment.arguments {
                        if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                            return inner_ty;
                        }
                    }
                }
            }
        }
    }

    ty
}

pub fn generate_field_to_string_expr(field: &Ident, ty: &Type) -> TokenStream {
    match ty {
        Type::ImplTrait(TypeImplTrait { .. }) => {
            // let is_into_string = bounds.iter().any(|b| {
            //     if let TypeParamBound::Trait(tb) = b {
            //         tb.path
            //             .segments
            //             .last()
            //             .map_or(false, |seg| seg.ident == "Into")
            //             && tb.path.to_token_stream().to_string().contains("String")
            //     } else {
            //         false
            //     }
            // });

            // if is_into_string {
            //     return quote! {
            //         #field.into()
            //     };
            // }

            quote! {
                #field.into()
            }
        }

        // Type::Path(tp) => {
        //     if let Some(seg) = tp.path.segments.last() {
        //         if seg.ident == "String" || seg.ident == "str" {
        //             return quote! {
        //                 #field.into()
        //             };
        //         }
        //     }
        // }
        _ => {
            // Fallback for primitives
            quote! {
                #field.to_string()
            }
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum CustomType {
    PrimaryKey(Path, Path),
    ForeignKey(Path, usize, Path),
    Unique(Path),
    Other(Path),
}

pub fn parse_custom_type(mut ty: &Type) -> CustomType {
    ty = unwrap_option_or_self(ty);
    if let Type::Path(type_path) = ty {
        let segment = match type_path.path.segments.last() {
            Some(seg) => seg,
            None => return CustomType::Other(type_path.path.clone()),
        };

        let args = match &segment.arguments {
            PathArguments::AngleBracketed(a) => &a.args,
            _ => return CustomType::Other(type_path.path.clone()),
        };

        let name = segment.ident.to_string();

        let mut type_args = Vec::new();
        let mut const_args = Vec::new();

        for arg in args {
            match arg {
                GenericArgument::Type(ty) => type_args.push(ty),
                GenericArgument::Const(expr) => const_args.push(expr),
                _ => {}
            }
        }

        match name.as_str() {
            "PrimaryKey" if type_args.len() == 2 => {
                if let (Type::Path(entity), Type::Path(inner)) = (&type_args[0], &type_args[1]) {
                    return CustomType::PrimaryKey(entity.path.clone(), inner.path.clone());
                }
            }
            "ForeignKey" if type_args.len() == 2 && const_args.len() == 1 => {
                if let (Type::Path(entity), Expr::Lit(expr_lit), Type::Path(inner)) =
                    (&type_args[0], &const_args[0], &type_args[1])
                {
                    if let Lit::Int(lit_int) = &expr_lit.lit {
                        if let Ok(index) = lit_int.base10_parse::<usize>() {
                            return CustomType::ForeignKey(
                                entity.path.clone(),
                                index,
                                inner.path.clone(),
                            );
                        }
                    }
                }
            }
            "Unique" if type_args.len() == 1 => {
                if let Type::Path(inner) = &type_args[0] {
                    return CustomType::Unique(inner.path.clone());
                }
            }
            _ => {}
        }

        CustomType::Other(type_path.path.clone())
    } else {
        CustomType::Other(Path {
            leading_colon: None,
            segments: Default::default(),
        })
    }
}
