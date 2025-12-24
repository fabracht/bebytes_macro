use quote::quote;
use syn::AngleBracketedGenericArguments;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::consts::{Endianness, PRIMITIVES};

pub fn get_from_bytes_method(endianness: Endianness) -> proc_macro2::TokenStream {
    match endianness {
        Endianness::Big => quote! { from_be_bytes },
        Endianness::Little => quote! { from_le_bytes },
    }
}

pub fn get_to_bytes_method(endianness: Endianness) -> proc_macro2::TokenStream {
    match endianness {
        Endianness::Big => quote! { to_be_bytes },
        Endianness::Little => quote! { to_le_bytes },
    }
}

pub fn get_try_from_bytes_method(endianness: Endianness) -> proc_macro2::TokenStream {
    match endianness {
        Endianness::Big => quote! { try_from_be_bytes },
        Endianness::Little => quote! { try_from_le_bytes },
    }
}

pub fn get_encode_to_method(endianness: Endianness) -> proc_macro2::TokenStream {
    match endianness {
        Endianness::Big => quote! { encode_be_to },
        Endianness::Little => quote! { encode_le_to },
    }
}

/// Get the size of a primitive type in bytes
pub fn get_primitive_type_size(field_type: &syn::Type) -> Result<usize, syn::Error> {
    match field_type {
        syn::Type::Path(tp)
            if tp.path.is_ident("i8") || tp.path.is_ident("u8") || tp.path.is_ident("bool") =>
        {
            Ok(1)
        }
        syn::Type::Path(tp) if tp.path.is_ident("i16") || tp.path.is_ident("u16") => Ok(2),
        syn::Type::Path(tp)
            if tp.path.is_ident("i32") || tp.path.is_ident("u32") || tp.path.is_ident("f32") =>
        {
            Ok(4)
        }
        syn::Type::Path(tp)
            if tp.path.is_ident("i64") || tp.path.is_ident("u64") || tp.path.is_ident("f64") =>
        {
            Ok(8)
        }
        syn::Type::Path(tp) if tp.path.is_ident("i128") || tp.path.is_ident("u128") => Ok(16),
        syn::Type::Path(tp) if tp.path.is_ident("char") => Ok(4),
        _ => Err(syn::Error::new_spanned(field_type, "Unsupported type")),
    }
}

/// Get the maximum number of bits that can be stored in a primitive type
pub fn get_primitive_type_max_bits(field_type: &syn::Type) -> Result<usize, syn::Error> {
    match field_type {
        syn::Type::Path(tp) if tp.path.is_ident("i8") || tp.path.is_ident("u8") => Ok(8),
        syn::Type::Path(tp) if tp.path.is_ident("i16") || tp.path.is_ident("u16") => Ok(16),
        syn::Type::Path(tp) if tp.path.is_ident("i32") || tp.path.is_ident("u32") => Ok(32),
        syn::Type::Path(tp) if tp.path.is_ident("i64") || tp.path.is_ident("u64") => Ok(64),
        syn::Type::Path(tp) if tp.path.is_ident("i128") || tp.path.is_ident("u128") => Ok(128),
        syn::Type::Path(tp) if tp.path.is_ident("char") => Ok(32),
        _ => Err(syn::Error::new_spanned(
            field_type,
            "Unsupported type for bits attribute",
        )),
    }
}

pub fn solve_for_inner_type(input: &syn::TypePath, identifier: &str) -> Option<syn::Type> {
    let syn::TypePath {
        path: syn::Path { segments, .. },
        ..
    } = input;
    let args = match &segments[0] {
        syn::PathSegment {
            ident,
            arguments:
                syn::PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }),
        } if ident == identifier && args.len() == 1 => args,
        _ => return None,
    };

    let syn::GenericArgument::Type(inner_type) = &args[0] else {
        return None;
    };

    Some(inner_type.clone())
}

pub fn is_primitive_identity(ident: &syn::Ident) -> bool {
    PRIMITIVES.iter().any(|&primitive| ident == primitive)
}

pub fn is_primitive_type(tp: &syn::TypePath) -> bool {
    PRIMITIVES
        .iter()
        .any(|&primitive| tp.path.is_ident(primitive))
}

/// Check if a type is Vec<Vec<u8>>
pub fn is_vec_of_vec_u8(tp: &syn::TypePath) -> bool {
    // Check if outer type is Vec
    if let Some(segment) = tp.path.segments.first() {
        if segment.ident == "Vec" {
            // Check if inner type is also Vec<u8>
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                if let Some(syn::GenericArgument::Type(syn::Type::Path(inner_tp))) =
                    args.args.first()
                {
                    if let Some(inner_segment) = inner_tp.path.segments.first() {
                        if inner_segment.ident == "Vec" {
                            // Check if innermost type is u8
                            if let syn::PathArguments::AngleBracketed(inner_args) =
                                &inner_segment.arguments
                            {
                                if let Some(syn::GenericArgument::Type(syn::Type::Path(
                                    innermost_tp,
                                ))) = inner_args.args.first()
                                {
                                    return innermost_tp.path.is_ident("u8");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn test_get_primitive_type_size_all_types() {
        // Test all primitive types return correct sizes
        let test_cases = vec![
            // 1-byte types
            ("u8", 1),
            ("i8", 1),
            // 2-byte types
            ("u16", 2),
            ("i16", 2),
            // 4-byte types
            ("u32", 4),
            ("i32", 4),
            ("f32", 4),
            // 8-byte types
            ("u64", 8),
            ("i64", 8),
            ("f64", 8),
            // 16-byte types
            ("u128", 16),
            ("i128", 16),
        ];

        for (type_str, expected_size) in test_cases {
            let ident = syn::Ident::new(type_str, proc_macro2::Span::call_site());
            let ty: syn::Type = parse_quote!(#ident);
            let size = get_primitive_type_size(&ty).unwrap();
            assert_eq!(
                size, expected_size,
                "Type {type_str} should have size {expected_size}"
            );
        }
    }

    #[test]
    fn test_get_primitive_type_size_non_primitives() {
        // Non-primitive types should return errors
        let non_primitives = vec![
            parse_quote!(String),
            parse_quote!(Vec<u8>),
            parse_quote!(Option<u32>),
            parse_quote!(CustomType),
            parse_quote!([u8; 10]),
        ];

        for ty in non_primitives {
            assert!(
                get_primitive_type_size(&ty).is_err(),
                "Non-primitive type should return error"
            );
        }
    }

    #[test]
    fn test_is_primitive_type() {
        // Test primitive identification
        let primitives = vec![
            "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64",
        ];

        for prim in primitives {
            let ident = syn::Ident::new(prim, proc_macro2::Span::call_site());
            let ty: syn::Type = parse_quote!(#ident);
            if let syn::Type::Path(tp) = &ty {
                assert!(is_primitive_type(tp), "{prim} should be primitive");
            } else {
                panic!("Expected Type::Path for {prim}");
            }
        }

        // Test non-primitives
        let non_primitives = vec![
            parse_quote!(String),
            parse_quote!(Vec<u8>),
            parse_quote!(Option<u32>),
            parse_quote!(CustomStruct),
        ];

        for ty in non_primitives {
            if let syn::Type::Path(tp) = &ty {
                assert!(!is_primitive_type(tp), "Should not be primitive");
            } else {
                // Non-path types are definitely not primitives
                // Non-path type is not primitive
            }
        }
    }

    #[test]
    fn test_solve_for_inner_type() {
        // Test Option inner type extraction
        let opt_ty: syn::Type = parse_quote!(Option<u32>);
        if let syn::Type::Path(tp) = &opt_ty {
            let inner = solve_for_inner_type(tp, "Option").unwrap();
            assert_eq!(quote!(#inner).to_string(), "u32");
        }

        // Test Vec inner type extraction
        let vec_ty: syn::Type = parse_quote!(Vec<String>);
        if let syn::Type::Path(tp) = &vec_ty {
            let inner = solve_for_inner_type(tp, "Vec").unwrap();
            assert_eq!(quote!(#inner).to_string(), "String");
        }

        // Test non-matching type
        let other_ty: syn::Type = parse_quote!(HashMap<String, u32>);
        if let syn::Type::Path(tp) = &other_ty {
            assert!(solve_for_inner_type(tp, "Option").is_none());
        }
    }

    #[test]
    fn test_get_from_bytes_method() {
        use crate::consts::Endianness;

        let be_method = get_from_bytes_method(Endianness::Big);
        assert_eq!(be_method.to_string(), "from_be_bytes");

        let le_method = get_from_bytes_method(Endianness::Little);
        assert_eq!(le_method.to_string(), "from_le_bytes");
    }

    #[test]
    fn test_get_to_bytes_method() {
        use crate::consts::Endianness;

        let be_method = get_to_bytes_method(Endianness::Big);
        assert_eq!(be_method.to_string(), "to_be_bytes");

        let le_method = get_to_bytes_method(Endianness::Little);
        assert_eq!(le_method.to_string(), "to_le_bytes");
    }

    #[test]
    fn test_get_try_from_bytes_method() {
        use crate::consts::Endianness;

        let be_method = get_try_from_bytes_method(Endianness::Big);
        assert_eq!(be_method.to_string(), "try_from_be_bytes");

        let le_method = get_try_from_bytes_method(Endianness::Little);
        assert_eq!(le_method.to_string(), "try_from_le_bytes");
    }

    #[test]
    fn test_is_primitive_identity() {
        // Test identity primitives (actually none of the primitives are "identity")
        // The function seems to be checking if it's ANY primitive, not identity primitive
        let primitives = vec![
            "u8", "i8", "u16", "u32", "u64", "u128", "i16", "i32", "i64", "i128", "f32", "f64",
        ];
        for ty_str in primitives {
            let ident = syn::Ident::new(ty_str, proc_macro2::Span::call_site());
            assert!(
                is_primitive_identity(&ident),
                "{ty_str} should be a primitive"
            );
        }

        // Test non-primitive types
        let non_primitives = vec!["String", "Vec"];
        for ty_str in non_primitives {
            let ident = syn::Ident::new(ty_str, proc_macro2::Span::call_site());
            assert!(
                !is_primitive_identity(&ident),
                "{ty_str} should not be a primitive"
            );
        }
    }

    #[test]
    fn test_edge_cases() {
        let empty_path: syn::Type = parse_quote!(::std::vec::Vec<u8>);
        if let syn::Type::Path(tp) = &empty_path {
            assert!(!is_primitive_type(tp));
        }
    }
}
