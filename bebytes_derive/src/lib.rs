//! `BeBytes` derive macro for binary serialization with bit fields and marker attributes.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::pedantic)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod attrs;
mod bit_validation;
mod consts;
mod enums;
mod functional;
mod optimization;
mod raw_pointer;
mod size_expr;
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

/// Calculate the total size of a struct based on its fields
fn calculate_struct_size(fields: &syn::FieldsNamed) -> Option<usize> {
    let mut total_size = 0usize;

    for field in &fields.named {
        let field_type = &field.ty;

        // Check if this is a bit field (has #[bits(N)] attribute)
        let is_bit_field = field.attrs.iter().any(|attr| attr.path().is_ident("bits"));
        if is_bit_field {
            return None;
        }

        // Try to get size of primitive types and arrays
        if let Ok(size) = crate::utils::get_primitive_type_size(field_type) {
            total_size += size;
        } else if let syn::Type::Array(array_type) = field_type {
            if let syn::Type::Path(element_type) = &*array_type.elem {
                if element_type.path.is_ident("u8") {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(len),
                        ..
                    }) = &array_type.len
                    {
                        if let Ok(array_len) = len.base10_parse::<usize>() {
                            total_size += array_len;
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    if total_size > 0 && total_size <= 256 {
        Some(total_size)
    } else {
        None
    }
}

/// Generate the "not supported" response for raw pointer methods
fn generate_raw_pointer_not_supported() -> proc_macro2::TokenStream {
    quote! {
        /// Check if this struct supports raw pointer encoding
        pub const fn supports_raw_pointer_encoding() -> bool {
            false
        }
    }
}

/// Generate raw pointer methods for ultra-high-performance encoding
/// These methods bypass all abstractions and write directly to memory
fn generate_raw_pointer_methods(
    fields: &syn::FieldsNamed,
    has_bit_fields: bool,
) -> proc_macro2::TokenStream {
    // Only generate for structs without bit fields
    if has_bit_fields {
        return generate_raw_pointer_not_supported();
    }

    // Calculate struct size
    let Some(total_size) = calculate_struct_size(fields) else {
        return generate_raw_pointer_not_supported();
    };

    // Generate the raw pointer writing code for big-endian
    let Ok(be_writing) = raw_pointer::generate_raw_pointer_struct_writing(fields, Endianness::Big)
    else {
        return generate_raw_pointer_not_supported();
    };

    // Generate the raw pointer writing code for little-endian
    let Ok(le_writing) =
        raw_pointer::generate_raw_pointer_struct_writing(fields, Endianness::Little)
    else {
        return generate_raw_pointer_not_supported();
    };

    quote! {
        /// Check if this struct supports raw pointer encoding
        pub const fn supports_raw_pointer_encoding() -> bool {
            true
        }

        /// Get the compile-time known size of this struct
        pub const RAW_POINTER_SIZE: usize = #total_size;

        /// Encode to a stack-allocated array using raw pointer operations (big-endian)
        /// This is the fastest possible encoding method with zero allocations
        ///
        /// # Compile-time Safety
        /// The array size is determined at compile time based on struct fields.
        #[inline(always)]
        pub fn encode_be_to_raw_stack(&self) -> [u8; #total_size] {
            let mut result = [0u8; #total_size];
            unsafe {
                let ptr = result.as_mut_ptr();
                let mut offset = 0;
                #be_writing
            }
            result
        }

        /// Encode to a stack-allocated array using raw pointer operations (little-endian)
        /// This is the fastest possible encoding method with zero allocations
        ///
        /// # Compile-time Safety
        /// The array size is determined at compile time based on struct fields.
        #[inline(always)]
        pub fn encode_le_to_raw_stack(&self) -> [u8; #total_size] {
            let mut result = [0u8; #total_size];
            unsafe {
                let ptr = result.as_mut_ptr();
                let mut offset = 0;
                #le_writing
            }
            result
        }

        /// Encode directly to a mutable buffer using raw pointer operations (big-endian)
        /// This method is unsafe and requires the buffer to have sufficient capacity
        #[inline(always)]
        pub unsafe fn encode_be_to_raw_mut<B: ::bebytes::BufMut>(&self, buf: &mut B) -> ::core::result::Result<(), ::bebytes::BeBytesError> {
            let required_capacity = Self::field_size();
            if buf.remaining_mut() < required_capacity {
                return Err(::bebytes::BeBytesError::InsufficientData {
                    expected: required_capacity,
                    actual: buf.remaining_mut(),
                });
            }

            let ptr = buf.chunk_mut().as_mut_ptr();
            let mut offset = 0;
            #be_writing
            buf.advance_mut(required_capacity);
            Ok(())
        }

        /// Encode directly to a mutable buffer using raw pointer operations (little-endian)
        /// This method is unsafe and requires the buffer to have sufficient capacity
        #[inline(always)]
        pub unsafe fn encode_le_to_raw_mut<B: ::bebytes::BufMut>(&self, buf: &mut B) -> ::core::result::Result<(), ::bebytes::BeBytesError> {
            let required_capacity = Self::field_size();
            if buf.remaining_mut() < required_capacity {
                return Err(::bebytes::BeBytesError::InsufficientData {
                    expected: required_capacity,
                    actual: buf.remaining_mut(),
                });
            }

            let ptr = buf.chunk_mut().as_mut_ptr();
            let mut offset = 0;
            #le_writing
            buf.advance_mut(required_capacity);
            Ok(())
        }
    }
}

/// Generate optimized direct writing methods for structs with bit fields
/// Uses stack-allocated arrays when possible to reduce allocation overhead
fn generate_bit_field_optimized_methods(
    _struct_field_names: &[&Option<syn::Ident>],
    _named_fields: &[proc_macro2::TokenStream],
    _le_named_fields: &[proc_macro2::TokenStream],
    _be_field_writing: &[proc_macro2::TokenStream],
    _le_field_writing: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
    quote! {
        #[inline]
        fn encode_be_to<B: ::bebytes::BufMut>(&self, buf: &mut B) -> ::core::result::Result<(), ::bebytes::BeBytesError> {
            let required_capacity = Self::field_size();
            if buf.remaining_mut() < required_capacity {
                return Err(::bebytes::BeBytesError::InsufficientData {
                    expected: required_capacity,
                    actual: buf.remaining_mut(),
                });
            }

            // For bit field structs, use existing to_be_bytes implementation
            // (Future optimization: implement true zero-allocation for small structs)
            let field_bytes = self.to_be_bytes();
            buf.put_slice(&field_bytes);

            Ok(())
        }

        #[inline]
        fn encode_le_to<B: ::bebytes::BufMut>(&self, buf: &mut B) -> ::core::result::Result<(), ::bebytes::BeBytesError> {
            let required_capacity = Self::field_size();
            if buf.remaining_mut() < required_capacity {
                return Err(::bebytes::BeBytesError::InsufficientData {
                    expected: required_capacity,
                    actual: buf.remaining_mut(),
                });
            }

            // For bit field structs, use existing to_le_bytes implementation
            // (Future optimization: implement true zero-allocation for small structs)
            let field_bytes = self.to_le_bytes();
            buf.put_slice(&field_bytes);

            Ok(())
        }
    }
}

#[allow(clippy::too_many_lines)]
#[proc_macro_derive(BeBytes, attributes(bits, With, FromField, bebytes, UntilMarker, AfterMarker))]
pub fn derive_be_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();
    let my_trait_path: syn::Path = syn::parse_quote!(::bebytes::BeBytes);

    let mut field_limit_check = Vec::new();

    let mut errors = Vec::new();

    // For big-endian implementation
    let mut be_field_parsing = Vec::new();
    let mut be_field_writing = Vec::new();
    let mut be_direct_writing = Vec::new();
    let mut has_bit_fields = false;

    // For little-endian implementation
    let mut le_field_parsing = Vec::new();
    let mut le_field_writing = Vec::new();
    let mut le_direct_writing: Vec<proc_macro2::TokenStream> = Vec::new();

    // Common elements
    let mut bit_sum = Vec::new();
    let mut named_fields = Vec::new();

    match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                let struct_field_names = fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();

                // Analyze struct for optimization opportunities
                let optimization_analysis = optimization::StructAnalysis::analyze_struct(&fields);

                // Generate big-endian implementation
                let mut be_context = structs::StructContext {
                    field_limit_check: &mut field_limit_check,
                    errors: &mut errors,
                    field_parsing: &mut be_field_parsing,
                    bit_sum: &mut bit_sum,
                    field_writing: &mut be_field_writing,
                    direct_writing: &mut be_direct_writing,
                    named_fields: &mut named_fields,
                    fields: &fields,
                    endianness: Endianness::Big,
                    has_bit_fields: &mut has_bit_fields,
                };
                structs::handle_struct(&mut be_context);

                // Generate little-endian implementation
                // field_limit_check and bit_sum are endian-independent, so we don't need to regenerate them
                // but named_fields needs to be regenerated for little-endian
                let mut le_named_fields = Vec::new();
                let mut le_dummy_field_limit = Vec::new(); // Dummy vector since we don't need to populate it again
                let mut le_dummy_bit_sum = Vec::new(); // Dummy vector since we don't need to populate it again
                let mut _le_dummy_direct_writing: Vec<proc_macro2::TokenStream> = Vec::new(); // Dummy vector since we use the shared direct_writing
                let mut le_dummy_has_bit_fields = false; // Dummy since bit fields are endian-independent
                let mut le_context = structs::StructContext {
                    field_limit_check: &mut le_dummy_field_limit,
                    errors: &mut errors,
                    field_parsing: &mut le_field_parsing,
                    bit_sum: &mut le_dummy_bit_sum,
                    field_writing: &mut le_field_writing,
                    direct_writing: &mut le_direct_writing,
                    named_fields: &mut le_named_fields,
                    fields: &fields,
                    endianness: Endianness::Little,
                    has_bit_fields: &mut le_dummy_has_bit_fields,
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

                // Generate direct writing methods for all structs
                // Bit field structs get stack-allocated optimization when possible
                let direct_writing_methods = if has_bit_fields {
                    // For structs with bit fields, generate optimized fallback methods
                    generate_bit_field_optimized_methods(
                        &struct_field_names,
                        &named_fields,
                        &le_named_fields,
                        &be_field_writing,
                        &le_field_writing,
                    )
                } else {
                    quote! {
                        #[inline]
                        fn encode_be_to<B: ::bebytes::BufMut>(&self, buf: &mut B) -> ::core::result::Result<(), ::bebytes::BeBytesError> {
                            let required_capacity = Self::field_size();
                            if buf.remaining_mut() < required_capacity {
                                return Err(::bebytes::BeBytesError::InsufficientData {
                                    expected: required_capacity,
                                    actual: buf.remaining_mut(),
                                });
                            }

                            let mut _bit_sum = 0;
                            #(
                                #named_fields
                                #be_direct_writing
                            )*
                            Ok(())
                        }

                        #[inline]
                        fn encode_le_to<B: ::bebytes::BufMut>(&self, buf: &mut B) -> ::core::result::Result<(), ::bebytes::BeBytesError> {
                            let required_capacity = Self::field_size();
                            if buf.remaining_mut() < required_capacity {
                                return Err(::bebytes::BeBytesError::InsufficientData {
                                    expected: required_capacity,
                                    actual: buf.remaining_mut(),
                                });
                            }

                            let mut _bit_sum = 0;
                            #(
                                #le_named_fields
                                #le_direct_writing
                            )*
                            Ok(())
                        }
                    }
                };

                // Generate raw pointer methods for eligible structs
                let raw_pointer_methods = generate_raw_pointer_methods(&fields, has_bit_fields);

                // Generate optimization methods
                let performance_docs = optimization_analysis.generate_performance_docs();
                let optimization_method_hint = optimization_analysis.generate_optimal_method_hint();
                let smart_method_selection =
                    optimization::generate_smart_method_selection(&optimization_analysis);
                let buffer_reuse_helpers = optimization::generate_buffer_reuse_helpers();
                let expanded = quote! {
                    #performance_docs
                    impl #my_trait_path for #name {
                        #[inline(always)]
                        fn field_size() -> usize {
                            let mut bit_sum = 0;
                            #(#bit_sum)*
                            bit_sum / 8
                        }

                        // Big-endian implementation
                        #[inline]
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

                        #[inline]
                        fn to_be_bytes(&self) -> Vec<u8> {
                            let capacity = Self::field_size();
                            let mut buf = ::bebytes::BytesMut::with_capacity(capacity);
                            let mut _bit_sum = 0;
                            #(
                                #named_fields
                                {
                                    let bytes = &mut buf;
                                    #be_field_writing
                                }
                            )*
                            buf.to_vec()
                        }

                        /// Convert to big-endian bytes as a zero-copy Bytes buffer
                        #[inline]
                        fn to_be_bytes_buf(&self) -> ::bebytes::Bytes {
                            let capacity = Self::field_size();
                            let mut buf = ::bebytes::BytesMut::with_capacity(capacity);
                            let mut _bit_sum = 0;
                            #(
                                #named_fields
                                {
                                    let bytes = &mut buf;
                                    #be_field_writing
                                }
                            )*
                            buf.freeze()
                        }

                        // Little-endian implementation
                        #[inline]
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

                        #[inline]
                        fn to_le_bytes(&self) -> Vec<u8> {
                            let capacity = Self::field_size();
                            let mut buf = ::bebytes::BytesMut::with_capacity(capacity);
                            let mut _bit_sum = 0;
                            #(
                                #le_named_fields
                                {
                                    let bytes = &mut buf;
                                    #le_field_writing
                                }
                            )*
                            buf.to_vec()
                        }

                        /// Convert to little-endian bytes as a zero-copy Bytes buffer
                        #[inline]
                        fn to_le_bytes_buf(&self) -> ::bebytes::Bytes {
                            let capacity = Self::field_size();
                            let mut buf = ::bebytes::BytesMut::with_capacity(capacity);
                            let mut _bit_sum = 0;
                            #(
                                #le_named_fields
                                {
                                    let bytes = &mut buf;
                                    #le_field_writing
                                }
                            )*
                            buf.freeze()
                        }

                        // Direct buffer writing methods (conditionally generated)
                        #direct_writing_methods

                    }

                    impl #name {
                        #[allow(clippy::too_many_arguments)]
                        pub fn new(#(#constructor_arg_list,)*) -> Self {
                            #(#field_limit_check)*
                            Self {
                                #( #struct_field_names, )*
                            }
                        }

                        // Raw pointer methods for ultra-high-performance encoding
                        #raw_pointer_methods

                        // Performance optimization methods
                        #optimization_method_hint

                        // Smart method selection for optimal performance
                        #smart_method_selection

                        // Buffer reuse helpers for batch operations
                        #buffer_reuse_helpers
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
                    #[inline(always)]
                    fn field_size() -> usize {
                        1
                    }

                    // Big-endian implementation
                    #[inline]
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

                    #[inline]
                    fn to_be_bytes(&self) -> Vec<u8> {
                        let mut buf = ::bebytes::BytesMut::with_capacity(1);
                        let val = match self {
                            #(#to_be_bytes_arms)*
                        };
                        ::bebytes::BufMut::put_u8(&mut buf, val);
                        buf.to_vec()
                    }

                    // Little-endian implementation
                    #[inline]
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

                    #[inline]
                    fn to_le_bytes(&self) -> Vec<u8> {
                        let mut buf = ::bebytes::BytesMut::with_capacity(1);
                        let val = match self {
                            #(#to_le_bytes_arms)*
                        };
                        ::bebytes::BufMut::put_u8(&mut buf, val);
                        buf.to_vec()
                    }

                    /// Convert to big-endian bytes as a Bytes buffer
                    #[inline]
                    fn to_be_bytes_buf(&self) -> ::bebytes::Bytes {
                        ::bebytes::Bytes::from(self.to_be_bytes())
                    }

                    /// Convert to little-endian bytes as a Bytes buffer
                    #[inline]
                    fn to_le_bytes_buf(&self) -> ::bebytes::Bytes {
                        ::bebytes::Bytes::from(self.to_le_bytes())
                    }

                    /// Encode directly to a buffer in big-endian format
                    #[inline]
                    fn encode_be_to<B: ::bebytes::BufMut>(&self, buf: &mut B) -> ::core::result::Result<(), ::bebytes::BeBytesError> {
                        if buf.remaining_mut() < 1 {
                            return Err(::bebytes::BeBytesError::InsufficientData {
                                expected: 1,
                                actual: buf.remaining_mut(),
                            });
                        }
                        let val = match self {
                            #(#to_be_bytes_arms)*
                        };
                        buf.put_u8(val);
                        Ok(())
                    }

                    /// Encode directly to a buffer in little-endian format
                    #[inline]
                    fn encode_le_to<B: ::bebytes::BufMut>(&self, buf: &mut B) -> ::core::result::Result<(), ::bebytes::BeBytesError> {
                        if buf.remaining_mut() < 1 {
                            return Err(::bebytes::BeBytesError::InsufficientData {
                                expected: 1,
                                actual: buf.remaining_mut(),
                            });
                        }
                        let val = match self {
                            #(#to_le_bytes_arms)*
                        };
                        buf.put_u8(val);
                        Ok(())
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

                            /// Decompose a u8 value into individual flag variants
                            pub fn decompose(bits: u8) -> ::bebytes::Vec<Self> {
                                let mut flags = ::bebytes::Vec::new();
                                #(
                                    if bits & #variant_values == #variant_values && #variant_values != 0 {
                                        if let Ok(flag) = Self::try_from(#variant_values) {
                                            flags.push(flag);
                                        }
                                    }
                                )*
                                flags
                            }

                            /// Iterate over individual flag variants set in a u8 value
                            pub fn iter_flags(bits: u8) -> impl Iterator<Item = Self> {
                                [
                                    #(
                                        if bits & #variant_values == #variant_values && #variant_values != 0 {
                                            Self::try_from(#variant_values).ok()
                                        } else {
                                            None
                                        },
                                    )*
                                ]
                                .into_iter()
                                .flatten()
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
