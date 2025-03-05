use quote::quote;
use syn::{spanned::Spanned, AngleBracketedGenericArguments};

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::consts::{PRIMITIVES, SUPPORTED_PRIMITIVES};

pub fn get_number_size(
    field_type: &syn::Type,
    field: &syn::Field,
    errors: &mut Vec<proc_macro2::TokenStream>,
) -> Option<usize> {
    let field_size = match &field_type {
        syn::Type::Path(tp) if tp.path.is_ident("i8") || tp.path.is_ident("u8") => 1,
        syn::Type::Path(tp) if tp.path.is_ident("i16") || tp.path.is_ident("u16") => 2,
        syn::Type::Path(tp)
            if tp.path.is_ident("i32") || tp.path.is_ident("u32") || tp.path.is_ident("f32") =>
        {
            4
        }
        syn::Type::Path(tp)
            if tp.path.is_ident("i64") || tp.path.is_ident("u64") || tp.path.is_ident("f64") =>
        {
            8
        }
        syn::Type::Path(tp) if tp.path.is_ident("i128") || tp.path.is_ident("u128") => 16,
        _ => {
            let error = syn::Error::new(field.ty.span(), "Unsupported type");
            errors.push(error.to_compile_error());
            return None;
        }
    };
    Some(field_size)
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
