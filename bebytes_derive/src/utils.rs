use quote::quote;
use syn::{spanned::Spanned, AngleBracketedGenericArguments};

#[cfg(feature = "std")]
use std::vec::Vec;

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

pub fn get_u8_bit_shift_direction(
    size: usize,
    pos: usize,
    endianness: Endianness,
) -> proc_macro2::TokenStream {
    match endianness {
        Endianness::Big => quote! { (7_usize - (#size + #pos % 8_usize - 1_usize)) },
        Endianness::Little => quote! { #pos % 8_usize },
    }
}

pub fn get_u8_bit_write_shift(
    size: usize,
    pos: usize,
    endianness: Endianness,
) -> proc_macro2::TokenStream {
    match endianness {
        Endianness::Big => quote! { (7_usize - (#size - 1_usize) - #pos % 8_usize) },
        Endianness::Little => quote! { #pos % 8_usize },
    }
}

pub fn get_number_size(
    field_type: &syn::Type,
    field: &syn::Field,
    errors: &mut Vec<proc_macro2::TokenStream>,
) -> Option<usize> {
    match get_primitive_type_size(field_type) {
        Ok(size) => Some(size),
        Err(_) => {
            let error = syn::Error::new(field.ty.span(), "Unsupported type");
            errors.push(error.to_compile_error());
            None
        }
    }
}

/// Get the size of a primitive type in bytes
pub fn get_primitive_type_size(field_type: &syn::Type) -> Result<usize, syn::Error> {
    match field_type {
        syn::Type::Path(tp) if tp.path.is_ident("i8") || tp.path.is_ident("u8") => Ok(1),
        syn::Type::Path(tp) if tp.path.is_ident("i16") || tp.path.is_ident("u16") => Ok(2),
        syn::Type::Path(tp)
            if tp.path.is_ident("i32") || tp.path.is_ident("u32") || tp.path.is_ident("f32") => Ok(4),
        syn::Type::Path(tp)
            if tp.path.is_ident("i64") || tp.path.is_ident("u64") || tp.path.is_ident("f64") => Ok(8),
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

    let inner_type = match &args[0] {
        syn::GenericArgument::Type(t) => t,
        _ => return None,
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

pub fn generate_chunks(n: usize, array_ident: proc_macro2::Ident) -> proc_macro2::TokenStream {
    let indices: Vec<_> = (0..n).map(|i| quote! { #array_ident[#i] }).collect();
    quote! { [ #( #indices ),* ] }
}

pub(crate) fn is_copy(field_type: &syn::Type) -> bool {
    match field_type {
        syn::Type::Never(_) => true, // ! is Copy
        syn::Type::Infer(_) => true, // _ is considered Copy for inference

        syn::Type::Path(type_path) => {
            // Check if it's a known Copy primitive or standard library type
            if let Some(ident) = type_path.path.get_ident() {
                let name = ident.to_string();
                match name.as_str() {
                    // Primitives that are Copy
                    "bool" | "char" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8"
                    | "i16" | "i32" | "i64" | "i128" | "isize" | "f32" | "f64" => true,

                    // Standard library types known to be Copy
                    "NonZero" | "NonZeroU8" | "NonZeroU16" | "NonZeroU32" | "NonZeroU64"
                    | "NonZeroU128" | "NonZeroUsize" | "NonZeroI8" | "NonZeroI16"
                    | "NonZeroI32" | "NonZeroI64" | "NonZeroI128" | "NonZeroIsize" => true,

                    // Types that are not Copy
                    "String" | "Vec" | "Box" | "Rc" | "Arc" | "RefCell" | "Cell" => false,

                    // For other types, you'd need more sophisticated analysis
                    // This might involve parsing attributes or checking trait bounds
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
        // These are generally not Copy
        syn::Type::BareFn(_) => false,
        syn::Type::ImplTrait(_) => false,
        syn::Type::Macro(_) => false,
        syn::Type::Ptr(_) => false, // Raw pointers are Copy, but we're being conservative
        syn::Type::Slice(_) => false, // Slices are not sized, so not Copy
        syn::Type::TraitObject(_) => false,
        syn::Type::Verbatim(_) => false,
        _ => false, // Conservative default for any other types
    }
}
