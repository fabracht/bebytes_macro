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

#[proc_macro_derive(BeBytes, attributes(U8, With, FromField))]
pub fn derive_be_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();
    let my_trait_path: syn::Path = syn::parse_str("BeBytes").unwrap();
    let mut field_limit_check = Vec::new();

    let mut errors = Vec::new();
    let mut field_parsing = Vec::new();
    let mut bit_sum = Vec::new();
    let mut field_writing = Vec::new();
    let mut named_fields = Vec::new();

    match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                let struct_field_names = fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();

                let total_size: usize = 0;
                let last_field = fields.named.last();

                structs::handle_struct(structs::StructContext {
                    field_limit_check: &mut field_limit_check,
                    errors: &mut errors,
                    field_parsing: &mut field_parsing,
                    bit_sum: &mut bit_sum,
                    field_writing: &mut field_writing,
                    named_fields: &mut named_fields,
                    fields: &fields,
                    total_size,
                    last_field,
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
                        fn try_from_be_bytes(bytes: &[u8]) -> Result<(Self, usize), Box<dyn std::error::Error>> {
                            if bytes.is_empty() {
                                return Err("No bytes provided.".into());
                            }

                            let mut _bit_sum = 0;
                            let mut byte_index = 0;
                            let mut end_byte_index = 0;
                            let buffer_size = bytes.len();
                            #(#field_parsing)*
                            Ok((Self {
                                #( #struct_field_names, )*
                            }, _bit_sum / 8))
                        }

                        fn to_be_bytes(&self) -> Vec<u8> {
                            let mut bytes = Vec::with_capacity(256);
                            let mut _bit_sum = 0;
                            #(
                                #named_fields
                                #field_writing
                            )*
                            bytes
                        }

                        fn field_size() -> usize {
                            let mut bit_sum = 0;
                            #(#bit_sum)*
                            bit_sum / 8
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
            let (from_be_bytes_arms, to_be_bytes_arms) = enums::handle_enum(errors, data_enum);

            let expanded = quote! {
                impl #my_trait_path for #name {
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

                    fn field_size() -> usize {
                        core::mem::size_of::<Self>()
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
