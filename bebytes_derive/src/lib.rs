#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod attrs;
mod bit_validation;
mod consts;
mod enums;
mod structs;
mod utils;

use proc_macro::TokenStream;
use quote::{__private::Span, quote};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields};

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use consts::Endianness;

// Define type aliases to reduce type complexity
type TokenVec = Vec<proc_macro2::TokenStream>;

// A struct to hold all the token collections
struct StructGenerationData {
    field_limit_check: TokenVec,
    be_field_parsing: TokenVec,
    be_field_writing: TokenVec,
    bit_sum: TokenVec,
    named_fields: TokenVec,
    le_field_parsing: TokenVec,
    le_field_writing: TokenVec,
    le_named_fields: TokenVec,
}

#[allow(clippy::too_many_arguments)]
fn generate_struct_expanded_impl(
    name: &syn::Ident,
    struct_field_names: &[&Option<syn::Ident>],
    gen_data: &StructGenerationData,
    constructor_arg_list: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
    let my_trait_path: syn::Path = syn::parse_str("BeBytes").unwrap();
    let bit_sum = &gen_data.bit_sum;
    let be_field_parsing = &gen_data.be_field_parsing;
    let named_fields = &gen_data.named_fields;
    let be_field_writing = &gen_data.be_field_writing;
    let le_field_parsing = &gen_data.le_field_parsing;
    let le_named_fields = &gen_data.le_named_fields;
    let le_field_writing = &gen_data.le_field_writing;
    let field_limit_check = &gen_data.field_limit_check;

    quote! {
        impl #my_trait_path for #name {
            fn field_size() -> usize {
                let mut bit_sum = 0;
                #(#bit_sum)*
                bit_sum / 8
            }

            // Big-endian implementation
            fn try_from_be_bytes(bytes: &[u8]) -> Result<(Self, usize), Box<dyn std::error::Error>> {
                if bytes.is_empty() {
                    return Err("No bytes provided.".into());
                }

                let mut _bit_sum = 0;
                let mut byte_index = 0;
                let mut end_byte_index = 0;
                let buffer_size = bytes.len();
                #(#be_field_parsing)*
                Ok((Self {
                    #( #struct_field_names, )*
                }, _bit_sum / 8))
            }

            fn to_be_bytes(&self) -> Vec<u8> {
                let mut bytes = Vec::with_capacity(256);
                let mut _bit_sum = 0;
                #(
                    #named_fields
                    #be_field_writing
                )*
                bytes
            }

            // Little-endian implementation
            fn try_from_le_bytes(bytes: &[u8]) -> Result<(Self, usize), Box<dyn std::error::Error>> {
                if bytes.is_empty() {
                    return Err("No bytes provided.".into());
                }

                let mut _bit_sum = 0;
                let mut byte_index = 0;
                let mut end_byte_index = 0;
                let buffer_size = bytes.len();
                #(#le_field_parsing)*
                Ok((Self {
                    #( #struct_field_names, )*
                }, _bit_sum / 8))
            }

            fn to_le_bytes(&self) -> Vec<u8> {
                let mut bytes = Vec::with_capacity(256);
                let mut _bit_sum = 0;
                #(
                    #le_named_fields
                    #le_field_writing
                )*
                bytes
            }
        }

        impl #name {
            #[allow(clippy::too_many_arguments)]
            pub fn new(#(#constructor_arg_list,)*) -> Self {
                #(#field_limit_check)*
                Self {
                    #( #struct_field_names, )*
                }
            }
        }
    }
}

fn generate_enum_expanded_impl(
    name: &syn::Ident,
    from_be_bytes_arms: &[proc_macro2::TokenStream],
    to_be_bytes_arms: &[proc_macro2::TokenStream],
    from_le_bytes_arms: &[proc_macro2::TokenStream],
    to_le_bytes_arms: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
    let my_trait_path: syn::Path = syn::parse_str("BeBytes").unwrap();

    quote! {
        impl #my_trait_path for #name {
            fn field_size() -> usize {
                core::mem::size_of::<Self>()
            }

            // Big-endian implementation
            fn try_from_be_bytes(bytes: &[u8]) -> Result<(Self, usize), Box<dyn std::error::Error>> {
                if bytes.is_empty() {
                    return Err("No bytes provided.".into());
                }
                let value = bytes[0];
                match value {
                    #(#from_be_bytes_arms)*
                    _ => Err(format!("No matching variant found for value {}", value).into()),
                }
            }

            fn to_be_bytes(&self) -> Vec<u8> {
                let mut bytes = Vec::with_capacity(1);
                let val = match self {
                    #(#to_be_bytes_arms)*
                };
                bytes.push(val);
                bytes
            }

            // Little-endian implementation
            fn try_from_le_bytes(bytes: &[u8]) -> Result<(Self, usize), Box<dyn std::error::Error>> {
                if bytes.is_empty() {
                    return Err("No bytes provided.".into());
                }
                let value = bytes[0];
                match value {
                    #(#from_le_bytes_arms)*
                    _ => Err(format!("No matching variant found for value {}", value).into()),
                }
            }

            fn to_le_bytes(&self) -> Vec<u8> {
                let mut bytes = Vec::with_capacity(1);
                let val = match self {
                    #(#to_le_bytes_arms)*
                };
                bytes.push(val);
                bytes
            }
        }
    }
}

fn process_named_fields(
    fields: &syn::FieldsNamed,
    errors: &mut Vec<proc_macro2::TokenStream>,
) -> StructGenerationData {
    let mut gen_data = StructGenerationData {
        field_limit_check: Vec::new(),
        be_field_parsing: Vec::new(),
        be_field_writing: Vec::new(),
        bit_sum: Vec::new(),
        named_fields: Vec::new(),
        le_field_parsing: Vec::new(),
        le_field_writing: Vec::new(),
        le_named_fields: Vec::new(),
    };

    let total_size: usize = 0;
    let last_field = fields.named.last();

    // Extract field attributes to analyze relationships
    let field_attrs_map = attrs::extract_struct_field_attributes(fields, errors);

    // Generate big-endian implementation
    structs::handle_struct(&mut structs::StructContext {
        field_limit_check: &mut gen_data.field_limit_check,
        errors,
        field_parsing: &mut gen_data.be_field_parsing,
        bit_sum: &mut gen_data.bit_sum,
        field_writing: &mut gen_data.be_field_writing,
        named_fields: &mut gen_data.named_fields,
        fields,
        total_size,
        last_field,
        endianness: Endianness::Big,
        field_attrs_map: field_attrs_map.clone(),
    });

    // Generate little-endian implementation
    // We reuse bit_sum because it's the same
    structs::handle_struct(&mut structs::StructContext {
        field_limit_check: &mut Vec::new(), // Already generated
        errors,
        field_parsing: &mut gen_data.le_field_parsing,
        bit_sum: &mut Vec::new(), // Already generated
        field_writing: &mut gen_data.le_field_writing,
        named_fields: &mut gen_data.le_named_fields,
        fields,
        total_size,
        last_field,
        endianness: Endianness::Little,
        field_attrs_map,
    });

    gen_data
}

#[allow(clippy::missing_panics_doc)]
#[proc_macro_derive(BeBytes, attributes(U8, With, FromField))]
pub fn derive_be_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();
    let mut errors = Vec::new();

    match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                let struct_field_names = fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();

                let gen_data = process_named_fields(&fields, &mut errors);

                // If there are any errors, return them immediately without generating code
                if !errors.is_empty() {
                    return quote! {
                        #(#errors)*
                    }
                    .into();
                }

                let constructor_arg_list = fields
                    .named
                    .iter()
                    .map(|f| {
                        let field_ident = &f.ident;
                        let field_type = &f.ty;
                        quote! { #field_ident: #field_type }
                    })
                    .collect::<Vec<_>>();

                let expanded = generate_struct_expanded_impl(
                    &name,
                    &struct_field_names,
                    &gen_data,
                    &constructor_arg_list,
                );

                let output = quote! {
                    #expanded
                    #(#errors)*
                };

                output.into()
            }
            field => {
                let error = syn::Error::new(field.span(), "Only named fields are supported")
                    .to_compile_error();
                quote! {
                    #error
                }
                .into()
            }
        },
        Data::Enum(data_enum) => {
            let (from_be_bytes_arms, to_be_bytes_arms) =
                enums::handle_enum(errors.clone(), data_enum.clone());
            let (from_le_bytes_arms, to_le_bytes_arms) = enums::handle_enum(Vec::new(), data_enum);

            let expanded = generate_enum_expanded_impl(
                &name,
                &from_be_bytes_arms,
                &to_be_bytes_arms,
                &from_le_bytes_arms,
                &to_le_bytes_arms,
            );

            expanded.into()
        }
        Data::Union(_) => {
            let error =
                syn::Error::new(Span::call_site(), "Type is not supported").to_compile_error();
            quote! {
                #error
            }
            .into()
        }
    }
}
