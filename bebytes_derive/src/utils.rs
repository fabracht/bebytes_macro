use quote::quote;
use syn::AngleBracketedGenericArguments;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::consts::{Endianness, PRIMITIVES, SUPPORTED_PRIMITIVES};

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

/// Get the size of a primitive type in bytes
pub fn get_primitive_type_size(field_type: &syn::Type) -> Result<usize, syn::Error> {
    match field_type {
        syn::Type::Path(tp) if tp.path.is_ident("i8") || tp.path.is_ident("u8") => Ok(1),
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
        _ => Err(syn::Error::new_spanned(field_type, "Unsupported type")),
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

pub fn is_supported_primitive_type(tp: &syn::TypePath) -> bool {
    SUPPORTED_PRIMITIVES
        .iter()
        .any(|&primitive| tp.path.is_ident(primitive))
}

pub(crate) fn is_copy(field_type: &syn::Type) -> bool {
    match field_type {
        syn::Type::Never(_) | syn::Type::Infer(_) => true, // ! and _ are Copy

        syn::Type::Path(type_path) => {
            // Check if it's a known Copy primitive or standard library type
            if let Some(ident) = type_path.path.get_ident() {
                let name = ident.to_string();
                match name.as_str() {
                    // Types that are Copy
                    "bool" | "char" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8"
                    | "i16" | "i32" | "i64" | "i128" | "isize" | "f32" | "f64" | "NonZero"
                    | "NonZeroU8" | "NonZeroU16" | "NonZeroU32" | "NonZeroU64" | "NonZeroU128"
                    | "NonZeroUsize" | "NonZeroI8" | "NonZeroI16" | "NonZeroI32" | "NonZeroI64"
                    | "NonZeroI128" | "NonZeroIsize" => true,

                    _ => false, // Conservatively assume non-Copy
                }
            } else if !type_path.path.segments.is_empty() {
                // Handle generic types
                let last_segment = &type_path.path.segments.last().unwrap();
                match last_segment.ident.to_string().as_str() {
                    "Option" => {
                        // Option<T> is Copy if T is Copy
                        if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                            if !args.args.is_empty() {
                                if let syn::GenericArgument::Type(ty) = &args.args[0] {
                                    return is_copy(ty);
                                }
                            }
                        }
                        false
                    }
                    "Result" => {
                        // Result<T, E> is Copy if both T and E are Copy
                        if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                            if args.args.len() >= 2 {
                                if let syn::GenericArgument::Type(t) = &args.args[0] {
                                    if let syn::GenericArgument::Type(e) = &args.args[1] {
                                        return is_copy(t) && is_copy(e);
                                    }
                                }
                            }
                        }
                        false
                    }
                    // Add more cases for other generic types
                    _ => false, // Conservatively assume non-Copy
                }
            } else {
                false
            }
        }

        syn::Type::Array(type_array) => is_copy(&type_array.elem), // Array<T> is Copy if T is Copy
        syn::Type::Tuple(type_tuple) => {
            // A tuple is Copy if all its elements are Copy
            type_tuple.elems.iter().all(is_copy)
        }
        syn::Type::Paren(type_paren) => is_copy(&type_paren.elem),
        syn::Type::Group(type_group) => is_copy(&type_group.elem),

        syn::Type::Reference(type_reference) => {
            // &T is always Copy, &mut T is never Copy
            type_reference.mutability.is_none()
        }
        _ => false, // Conservative default for any other types
    }
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
                "Type {} should have size {}",
                type_str, expected_size
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
                assert!(is_primitive_type(tp), "{} should be primitive", prim);
            } else {
                panic!("Expected Type::Path for {}", prim);
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
                assert!(true, "Non-path type is not primitive");
            }
        }
    }

    #[test]
    fn test_is_supported_primitive_type() {
        // BeBytes supports most primitives except usize/isize
        let supported = vec![
            "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128",
        ];

        for prim in supported {
            let ident = syn::Ident::new(prim, proc_macro2::Span::call_site());
            let ty: syn::Type = parse_quote!(#ident);
            if let syn::Type::Path(tp) = &ty {
                assert!(
                    is_supported_primitive_type(tp),
                    "{} should be supported",
                    prim
                );
            } else {
                panic!("Expected Type::Path for {}", prim);
            }
        }

        // Unsupported types
        let unsupported = vec![
            parse_quote!(usize),
            parse_quote!(isize),
            parse_quote!(f32),
            parse_quote!(f64),
            parse_quote!(char),
            parse_quote!(bool),
            parse_quote!(String),
        ];

        for ty in unsupported {
            if let syn::Type::Path(tp) = &ty {
                assert!(!is_supported_primitive_type(tp), "Should not be supported");
            } else {
                // Non-path types are definitely not supported
                assert!(true, "Non-path type is not supported");
            }
        }
    }

    #[test]
    fn test_is_copy_trait() {
        // Types that implement Copy
        let copy_types = vec![
            parse_quote!(u32),
            parse_quote!(i64),
            parse_quote!([u8; 10]),
            parse_quote!(Option<u32>),
            parse_quote!((u32, u64)),
            parse_quote!(Result<u32, u32>),
        ];

        for ty in copy_types {
            assert!(is_copy(&ty), "Type should implement Copy");
        }

        // Types that don't implement Copy
        let non_copy_types = vec![
            parse_quote!(String),
            parse_quote!(Vec<u8>),
            parse_quote!(Option<String>),
            parse_quote!(Result<String, u32>),
        ];

        for ty in non_copy_types {
            assert!(!is_copy(&ty), "Type should not implement Copy");
        }
    }

    #[test]
    fn test_is_copy_nested_types() {
        // Test nested type checking
        assert!(is_copy(&parse_quote!(Option<Option<u32>>)));
        assert!(!is_copy(&parse_quote!(Option<Vec<u8>>)));
        assert!(is_copy(&parse_quote!(Result<[u8; 10], u32>)));
        assert!(!is_copy(&parse_quote!(Result<String, String>)));
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
                "{} should be a primitive",
                ty_str
            );
        }

        // Test non-primitive types
        let non_primitives = vec!["String", "Vec"];
        for ty_str in non_primitives {
            let ident = syn::Ident::new(ty_str, proc_macro2::Span::call_site());
            assert!(
                !is_primitive_identity(&ident),
                "{} should not be a primitive",
                ty_str
            );
        }
    }

    #[test]
    fn test_edge_cases() {
        // Test empty path segments
        let empty_path: syn::Type = parse_quote!(::std::vec::Vec<u8>);
        if let syn::Type::Path(tp) = &empty_path {
            assert!(!is_primitive_type(tp));
        }

        // Test parenthesized types
        let paren_ty: syn::Type = parse_quote!((u32));
        assert!(is_copy(&paren_ty));

        // Test reference types
        let ref_ty: syn::Type = parse_quote!(&u32);
        assert!(is_copy(&ref_ty));

        let mut_ref_ty: syn::Type = parse_quote!(&mut u32);
        assert!(!is_copy(&mut_ref_ty));
    }
}
