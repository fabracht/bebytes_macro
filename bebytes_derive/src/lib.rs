#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod attrs;
mod bit_validation;
mod consts;
mod enums;
mod structs;
mod utils;

use attrs::FieldAttributesMap;
use proc_macro::TokenStream;
use quote::{__private::Span, quote};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields};

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use consts::Endianness;

#[proc_macro_derive(BeBytes, attributes(U8, With, FromField))]
pub fn derive_be_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();
    let my_trait_path: syn::Path = syn::parse_str("BeBytes").unwrap();
    let metadata_trait_path: syn::Path = syn::parse_str("BeBytesMetadata").unwrap();
    let mut field_limit_check = Vec::new();

    let mut errors = Vec::new();

    // For big-endian implementation
    let mut be_field_parsing = Vec::new();
    let mut be_field_writing = Vec::new();

    // For little-endian implementation
    let mut le_field_parsing = Vec::new();
    let mut le_field_writing = Vec::new();

    // Common elements
    let mut bit_sum = Vec::new();
    let mut named_fields = Vec::new();

    match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                let struct_field_names = fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();

                let total_size: usize = 0;
                let last_field = fields.named.last();

                // Extract field attributes to analyze relationships
                let field_attrs_map = attrs::extract_struct_field_attributes(&fields, &mut errors);

                // Generate metadata implementation after field relationships have been extracted
                let metadata_impl =
                    generate_metadata_impl(&name, metadata_trait_path, field_attrs_map.clone());
                // Generate big-endian implementation
                structs::handle_struct(structs::StructContext {
                    field_limit_check: &mut field_limit_check,
                    errors: &mut errors,
                    field_parsing: &mut be_field_parsing,
                    bit_sum: &mut bit_sum,
                    field_writing: &mut be_field_writing,
                    named_fields: &mut named_fields,
                    fields: &fields,
                    total_size,
                    last_field,
                    endianness: Endianness::Big,
                    field_attrs_map: field_attrs_map.clone(),
                });

                // Generate little-endian implementation
                // We reuse named_fields and bit_sum because they're the same
                let mut le_named_fields = Vec::new();
                structs::handle_struct(structs::StructContext {
                    field_limit_check: &mut Vec::new(), // Already generated
                    errors: &mut errors,
                    field_parsing: &mut le_field_parsing,
                    bit_sum: &mut Vec::new(), // Already generated
                    field_writing: &mut le_field_writing,
                    named_fields: &mut le_named_fields,
                    fields: &fields,
                    total_size,
                    last_field,
                    endianness: Endianness::Little,
                    field_attrs_map,
                });

                // If there are any errors, return them immediately without generating code
                if !errors.is_empty() {
                    return quote! {
                        #(#errors)*
                    }
                    .into();
                }

                let constructor_arg_list = fields.named.iter().map(|f| {
                    let field_ident = &f.ident;
                    let field_type = &f.ty;
                    quote! { #field_ident: #field_type }
                });

                let expanded = quote! {
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

                    impl BeBytesWith for #name {
                        // Default implementations for the with_sizes methods
                        // These will use external size information when provided
                        fn try_from_be_bytes_with_sizes(
                            bytes: &[u8],
                            sizes: &std::collections::HashMap<&'static str, usize>,
                        ) -> Result<(Self, usize), Box<dyn std::error::Error>>
                        where
                            Self: Sized + BeBytesMetadata
                        {
                            Self::try_from_be_bytes(bytes)
                        }

                        fn try_from_le_bytes_with_sizes(
                            bytes: &[u8],
                            sizes: &std::collections::HashMap<&'static str, usize>,
                        ) -> Result<(Self, usize), Box<dyn std::error::Error>>
                        where
                            Self: Sized + BeBytesMetadata
                        {
                            Self::try_from_le_bytes(bytes)
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

                    #metadata_impl
                };

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
                enums::handle_enum(errors, data_enum.clone());
            let (from_le_bytes_arms, to_le_bytes_arms) = enums::handle_enum(Vec::new(), data_enum);

            let expanded = quote! {
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
            };
            expanded.into()
        }
        _ => {
            let error =
                syn::Error::new(Span::call_site(), "Type is not supported").to_compile_error();
            let output = quote! {
                #error
            };

            output.into()
        }
    }
}

fn generate_metadata_impl(
    name: &syn::Ident,
    metadata_trait_path: syn::Path,
    field_attrs_map: FieldAttributesMap,
) -> Option<proc_macro2::TokenStream> {
    // Create vectors to collect field mapping relationships
    let mut requires_sizes = Vec::new();

    // Iterate through the field attributes map to find ForField and FromField pairs
    for (field_name, attrs) in &field_attrs_map {
        // If this field has a FromField attribute (getting size from another field)
        if attrs.has_from_field() {
            // Add to the list of fields requiring external size information
            requires_sizes.push(quote! {
                sizes.insert(#field_name);
            });
        }
    }

    let metadata_impl = {
        Some(quote! {
            impl #metadata_trait_path for #name {
                fn requires_external_sizes() -> std::collections::HashSet<&'static str> {
                    let mut sizes = std::collections::HashSet::new();
                    #(#requires_sizes)*
                    sizes
                }
            }
        })
    };
    metadata_impl
}
