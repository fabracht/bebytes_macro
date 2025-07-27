#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::pedantic)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod attrs;
mod bit_validation;
mod consts;
mod enums;
mod functional;
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

#[allow(clippy::too_many_lines)]
#[proc_macro_derive(BeBytes, attributes(bits, With, FromField, bebytes))]
pub fn derive_be_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();
    let my_trait_path: syn::Path = syn::parse_quote!(::bebytes::BeBytes);

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

                // Generate big-endian implementation
                let mut be_context = structs::StructContext {
                    field_limit_check: &mut field_limit_check,
                    errors: &mut errors,
                    field_parsing: &mut be_field_parsing,
                    bit_sum: &mut bit_sum,
                    field_writing: &mut be_field_writing,
                    named_fields: &mut named_fields,
                    fields: &fields,
                    endianness: Endianness::Big,
                };
                structs::handle_struct(&mut be_context);

                // Generate little-endian implementation
                // field_limit_check and bit_sum are endian-independent, so we don't need to regenerate them
                // but named_fields needs to be regenerated for little-endian
                let mut le_named_fields = Vec::new();
                let mut le_dummy_field_limit = Vec::new(); // Dummy vector since we don't need to populate it again
                let mut le_dummy_bit_sum = Vec::new(); // Dummy vector since we don't need to populate it again
                let mut le_context = structs::StructContext {
                    field_limit_check: &mut le_dummy_field_limit,
                    errors: &mut errors,
                    field_parsing: &mut le_field_parsing,
                    bit_sum: &mut le_dummy_bit_sum,
                    field_writing: &mut le_field_writing,
                    named_fields: &mut le_named_fields,
                    fields: &fields,
                    endianness: Endianness::Little,
                };
                structs::handle_struct(&mut le_context);

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
                        fn try_from_be_bytes(bytes: &[u8]) -> ::core::result::Result<(Self, usize), ::bebytes::BeBytesError> {
                            if bytes.is_empty() {
                                return Err(::bebytes::BeBytesError::EmptyBuffer);
                            }

                            let mut _bit_sum = 0;
                            let mut byte_index = 0;
                            let mut end_byte_index = 0;
                            let buffer_size = bytes.len();
                            #(#be_field_parsing)*
                            Ok((Self {
                                #( #struct_field_names, )*
                            }, usize::div_ceil(_bit_sum as usize, 8)))
                        }

                        fn to_be_bytes(&self) -> Vec<u8> {
                            let capacity = Self::field_size();
                            let mut bytes = Vec::with_capacity(capacity);
                            let mut _bit_sum = 0;
                            #(
                                #named_fields
                                #be_field_writing
                            )*
                            bytes
                        }

                        // Little-endian implementation
                        fn try_from_le_bytes(bytes: &[u8]) -> ::core::result::Result<(Self, usize), ::bebytes::BeBytesError> {
                            if bytes.is_empty() {
                                return Err(::bebytes::BeBytesError::EmptyBuffer);
                            }

                            let mut _bit_sum = 0;
                            let mut byte_index = 0;
                            let mut end_byte_index = 0;
                            let buffer_size = bytes.len();
                            #(#le_field_parsing)*
                            Ok((Self {
                                #( #struct_field_names, )*
                            }, usize::div_ceil(_bit_sum as usize, 8)))
                        }

                        fn to_le_bytes(&self) -> Vec<u8> {
                            let capacity = Self::field_size();
                            let mut bytes = Vec::with_capacity(capacity);
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
            // Check if this is a flags enum
            let is_flags_enum = input.attrs.iter().any(|attr| {
                attr.path().is_ident("bebytes") && {
                    let mut is_flags = false;
                    if let Ok(()) = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("flags") {
                            is_flags = true;
                        }
                        Ok(())
                    }) {
                        is_flags
                    } else {
                        false
                    }
                }
            });

            let (
                from_be_bytes_arms,
                to_be_bytes_arms,
                _,
                try_from_arms,
                discriminants,
                enum_errors,
            ) = enums::handle_enum(Vec::new(), data_enum.clone());
            let (from_le_bytes_arms, to_le_bytes_arms, _, _, _, _) =
                enums::handle_enum(Vec::new(), data_enum);

            // If there are any errors from enum validation, return them
            if !enum_errors.is_empty() {
                return quote! {
                    #(#enum_errors)*
                }
                .into();
            }

            let expanded = quote! {
                impl ::core::convert::TryFrom<u8> for #name {
                    type Error = ::bebytes::BeBytesError;

                    fn try_from(value: u8) -> ::core::result::Result<Self, Self::Error> {
                        match value {
                            #(#try_from_arms)*
                            _ => Err(::bebytes::BeBytesError::InvalidDiscriminant {
                                value,
                                type_name: stringify!(#name),
                            }),
                        }
                    }
                }

                impl #my_trait_path for #name {
                    fn field_size() -> usize {
                        core::mem::size_of::<Self>()
                    }

                    // Big-endian implementation
                    fn try_from_be_bytes(bytes: &[u8]) -> ::core::result::Result<(Self, usize), ::bebytes::BeBytesError> {
                        if bytes.is_empty() {
                            return Err(::bebytes::BeBytesError::EmptyBuffer);
                        }
                        let value = bytes[0];
                        match value {
                            #(#from_be_bytes_arms)*
                            _ => Err(::bebytes::BeBytesError::InvalidDiscriminant {
                                value,
                                type_name: stringify!(#name),
                            }),
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
                    fn try_from_le_bytes(bytes: &[u8]) -> ::core::result::Result<(Self, usize), ::bebytes::BeBytesError> {
                        if bytes.is_empty() {
                            return Err(::bebytes::BeBytesError::EmptyBuffer);
                        }
                        let value = bytes[0];
                        match value {
                            #(#from_le_bytes_arms)*
                            _ => Err(::bebytes::BeBytesError::InvalidDiscriminant {
                                value,
                                type_name: stringify!(#name),
                            }),
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

            // Generate bitwise operations for flag enums
            let bitwise_ops = if is_flags_enum {
                // Validate that all discriminants are powers of 2
                let mut validation_errors = Vec::new();
                let variant_values = discriminants
                    .iter()
                    .map(|(_, value)| *value)
                    .collect::<Vec<_>>();

                for (ident, value) in &discriminants {
                    if *value != 0 && (*value & (*value - 1)) != 0 {
                        validation_errors.push(
                            syn::Error::new(
                                ident.span(),
                                format!(
                                    "Flag enum variant '{ident}' has value {value} which is not a power of 2"
                                ),
                            )
                            .to_compile_error(),
                        );
                    }
                }

                if validation_errors.is_empty() {
                    quote! {
                        #expanded

                        // Bitwise operations for flag enums
                        impl core::ops::BitOr for #name {
                            type Output = u8;

                            fn bitor(self, rhs: Self) -> Self::Output {
                                (self as u8) | (rhs as u8)
                            }
                        }

                        impl core::ops::BitOr<u8> for #name {
                            type Output = u8;

                            fn bitor(self, rhs: u8) -> Self::Output {
                                (self as u8) | rhs
                            }
                        }

                        impl core::ops::BitOr<#name> for u8 {
                            type Output = u8;

                            fn bitor(self, rhs: #name) -> Self::Output {
                                self | (rhs as u8)
                            }
                        }

                        impl core::ops::BitAnd for #name {
                            type Output = u8;

                            fn bitand(self, rhs: Self) -> Self::Output {
                                (self as u8) & (rhs as u8)
                            }
                        }

                        impl core::ops::BitAnd<u8> for #name {
                            type Output = u8;

                            fn bitand(self, rhs: u8) -> Self::Output {
                                (self as u8) & rhs
                            }
                        }

                        impl core::ops::BitAnd<#name> for u8 {
                            type Output = u8;

                            fn bitand(self, rhs: #name) -> Self::Output {
                                self & (rhs as u8)
                            }
                        }

                        impl core::ops::BitXor for #name {
                            type Output = u8;

                            fn bitxor(self, rhs: Self) -> Self::Output {
                                (self as u8) ^ (rhs as u8)
                            }
                        }

                        impl core::ops::BitXor<u8> for #name {
                            type Output = u8;

                            fn bitxor(self, rhs: u8) -> Self::Output {
                                (self as u8) ^ rhs
                            }
                        }

                        impl core::ops::BitXor<#name> for u8 {
                            type Output = u8;

                            fn bitxor(self, rhs: #name) -> Self::Output {
                                self ^ (rhs as u8)
                            }
                        }

                        impl core::ops::Not for #name {
                            type Output = u8;

                            fn not(self) -> Self::Output {
                                !(self as u8)
                            }
                        }

                        impl #name {
                            /// Check if this flag value contains the given flag
                            pub fn contains(self, flag: Self) -> bool {
                                ((self as u8) & (flag as u8)) == (flag as u8)
                            }

                            /// Create a flags value from a u8
                            pub fn from_bits(bits: u8) -> Option<u8> {
                                // Validate that all bits correspond to valid flags
                                let mut remaining = bits;
                                #(
                                    if (bits & #variant_values) == #variant_values {
                                        remaining &= !#variant_values;
                                    }
                                )*
                                if remaining == 0 {
                                    Some(bits)
                                } else {
                                    None
                                }
                            }
                        }
                    }
                } else {
                    quote! {
                        #expanded
                        #(#validation_errors)*
                    }
                }
            } else {
                expanded
            };

            bitwise_ops.into()
        }
        Data::Union(_) => {
            let error =
                syn::Error::new(Span::call_site(), "Type is not supported").to_compile_error();
            let output = quote! {
                #error
            };

            output.into()
        }
    }
}
