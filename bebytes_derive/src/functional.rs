use proc_macro2::TokenStream;
use syn::{Error as SynError, Ident};

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Error aggregation type for handling multiple parse errors
pub type ParseResult<T> = Result<T, Vec<SynError>>;

/// Processing context to thread through computations
#[derive(Clone, Debug)]
pub struct ProcessingContext {
    pub endianness: crate::consts::Endianness,
    pub bit_position: usize,
    pub is_last_field: bool,
}

impl ProcessingContext {
    pub fn new(endianness: crate::consts::Endianness) -> Self {
        Self {
            endianness,
            bit_position: 0,
            is_last_field: false,
        }
    }

    pub fn with_bit_position(mut self, bit_position: usize) -> Self {
        self.bit_position = bit_position;
        self
    }

    pub fn with_last_field(mut self, is_last_field: bool) -> Self {
        self.is_last_field = is_last_field;
        self
    }
}

/// Result of processing a single field
#[derive(Debug, Clone)]
pub struct FieldProcessResult {
    pub limit_check: TokenStream,
    pub parsing: TokenStream,
    pub writing: TokenStream,
    pub direct_writing: TokenStream, // New: direct buffer writing
    pub accessor: TokenStream,
    pub bit_sum: TokenStream,
}

impl FieldProcessResult {
    pub fn new(
        limit_check: TokenStream,
        parsing: TokenStream,
        writing: TokenStream,
        direct_writing: TokenStream,
        accessor: TokenStream,
        bit_sum: TokenStream,
    ) -> Self {
        Self {
            limit_check,
            parsing,
            writing,
            direct_writing,
            accessor,
            bit_sum,
        }
    }
}

/// Builder pattern for complex `FieldData` structures
pub struct FieldDataBuilder {
    limit_checks: Vec<TokenStream>,
    parsings: Vec<TokenStream>,
    writings: Vec<TokenStream>,
    direct_writings: Vec<TokenStream>, // New: direct buffer writings
    accessors: Vec<TokenStream>,
    bit_sums: Vec<TokenStream>,
}

impl FieldDataBuilder {
    pub fn new() -> Self {
        Self {
            limit_checks: Vec::new(),
            parsings: Vec::new(),
            writings: Vec::new(),
            direct_writings: Vec::new(),
            accessors: Vec::new(),
            bit_sums: Vec::new(),
        }
    }

    pub fn add_result(mut self, result: FieldProcessResult) -> Self {
        self.limit_checks.push(result.limit_check);
        self.parsings.push(result.parsing);
        self.writings.push(result.writing);
        self.direct_writings.push(result.direct_writing);
        self.accessors.push(result.accessor);
        self.bit_sums.push(result.bit_sum);
        self
    }

    pub fn build(self) -> crate::structs::FieldData {
        crate::structs::FieldData {
            field_limit_check: self.limit_checks,
            errors: Vec::new(), // Errors handled separately now
            field_parsing: self.parsings,
            bit_sum: self.bit_sums,
            field_writing: self.writings,
            direct_writing: self.direct_writings,
            named_fields: self.accessors,
            total_size: 0,
        }
    }
}

impl Default for FieldDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Attribute data structure for functional parsing
#[derive(Debug, Default, Clone)]
pub struct AttributeData {
    pub size: Option<usize>,
    pub field: Option<Vec<Ident>>,
    pub size_expression: Option<crate::size_expr::SizeExpression>,
    pub is_bits_attribute: bool,
}

impl AttributeData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_field(mut self, field: Vec<Ident>) -> Self {
        self.field = Some(field);
        self
    }

    pub fn with_bits_attribute(mut self) -> Self {
        self.is_bits_attribute = true;
        self
    }

    pub fn with_size_expression(mut self, size_expr: crate::size_expr::SizeExpression) -> Self {
        self.size_expression = Some(size_expr);
        self
    }

    /// Merge multiple `AttributeData` instances, prioritizing non-`None` values
    pub fn merge(attrs: Vec<Self>) -> Self {
        attrs.into_iter().fold(Self::default(), |mut acc, attr| {
            acc.size = attr.size.or(acc.size);
            acc.field = attr.field.or(acc.field);
            acc.size_expression = attr.size_expression.or(acc.size_expression);
            acc.is_bits_attribute |= attr.is_bits_attribute;
            acc
        })
    }
}

/// Error aggregation utilities
pub mod error_utils {
    use super::{ParseResult, SynError, Vec};

    /// Collect results, separating successes from errors
    pub fn aggregate_results<T>(
        results: impl Iterator<Item = Result<T, SynError>>,
    ) -> ParseResult<Vec<T>> {
        let results: Vec<_> = results.collect();
        let mut successes = Vec::new();
        let mut errors = Vec::new();

        for result in results {
            match result {
                Ok(success) => successes.push(success),
                Err(error) => errors.push(error),
            }
        }

        if errors.is_empty() {
            Ok(successes)
        } else {
            Err(errors)
        }
    }
}

/// Pure helper functions to replace mutation-based ones
pub mod pure_helpers {
    use super::TokenStream;
    use quote::quote;
    use syn::Ident;

    /// Create a field accessor without side effects
    pub fn create_field_accessor(field_name: &Ident, needs_owned: bool) -> TokenStream {
        if needs_owned {
            quote! { let #field_name = self.#field_name.clone(); }
        } else {
            quote! { let #field_name = self.#field_name; }
        }
    }

    /// Create a bit sum token without side effects
    pub fn create_bit_sum(size: usize) -> TokenStream {
        quote! { bit_sum += #size; }
    }

    /// Create a bit sum for byte-aligned fields
    pub fn create_byte_bit_sum(size: usize) -> TokenStream {
        quote! { bit_sum += #size * 8; }
    }

    /// Create byte indices tokens
    pub fn create_byte_indices(field_size: usize) -> TokenStream {
        quote! {
            // Ensure byte alignment when transitioning from bitfields
            if _bit_sum % 8 != 0 {
                _bit_sum = usize::div_ceil(_bit_sum, 8) * 8;
            }
            byte_index = _bit_sum / 8;
            end_byte_index = byte_index + #field_size;
            if end_byte_index > bytes.len() {
                return Err(::bebytes::BeBytesError::InsufficientData {
                    expected: #field_size,
                    actual: bytes.len().saturating_sub(byte_index),
                });
            }
            _bit_sum += 8 * #field_size;
        }
    }

    /// Create primitive type parsing code
    pub fn create_primitive_parsing(
        field_name: &Ident,
        field_type: &syn::Type,
        endianness: crate::consts::Endianness,
    ) -> Result<TokenStream, syn::Error> {
        let type_size = crate::utils::get_primitive_type_size(field_type)?;

        // Special handling for char type
        if let syn::Type::Path(tp) = field_type {
            if tp.path.is_ident("char") {
                return Ok(create_char_parsing(field_name, endianness));
            }
        }

        create_numeric_parsing(field_name, field_type, type_size, endianness)
    }

    /// Create char type parsing code
    fn create_char_parsing(
        field_name: &Ident,
        endianness: crate::consts::Endianness,
    ) -> TokenStream {
        match endianness {
            crate::consts::Endianness::Big => quote! {
                let char_value = u32::from_be_bytes([
                    bytes[byte_index], bytes[byte_index + 1],
                    bytes[byte_index + 2], bytes[byte_index + 3]
                ]);
                let #field_name = char::from_u32(char_value)
                    .ok_or_else(|| ::bebytes::BeBytesError::InvalidDiscriminant {
                        value: (char_value & 0xFF) as u8,
                        type_name: "char",
                    })?;
            },
            crate::consts::Endianness::Little => quote! {
                let char_value = u32::from_le_bytes([
                    bytes[byte_index], bytes[byte_index + 1],
                    bytes[byte_index + 2], bytes[byte_index + 3]
                ]);
                let #field_name = char::from_u32(char_value)
                    .ok_or_else(|| ::bebytes::BeBytesError::InvalidDiscriminant {
                        value: (char_value & 0xFF) as u8,
                        type_name: "char",
                    })?;
            },
        }
    }

    /// Create numeric type parsing code
    fn create_numeric_parsing(
        field_name: &Ident,
        field_type: &syn::Type,
        type_size: usize,
        endianness: crate::consts::Endianness,
    ) -> Result<TokenStream, syn::Error> {
        let from_bytes_method = match endianness {
            crate::consts::Endianness::Big => quote!(from_be_bytes),
            crate::consts::Endianness::Little => quote!(from_le_bytes),
        };

        match type_size {
            1 => Ok(quote! {
                let #field_name = bytes[byte_index] as #field_type;
            }),
            2 => Ok(quote! {
                let #field_name = #field_type::#from_bytes_method([
                    bytes[byte_index], bytes[byte_index + 1]
                ]);
            }),
            4 => Ok(quote! {
                let #field_name = #field_type::#from_bytes_method([
                    bytes[byte_index], bytes[byte_index + 1],
                    bytes[byte_index + 2], bytes[byte_index + 3]
                ]);
            }),
            8 => Ok(quote! {
                let #field_name = #field_type::#from_bytes_method([
                    bytes[byte_index], bytes[byte_index + 1],
                    bytes[byte_index + 2], bytes[byte_index + 3],
                    bytes[byte_index + 4], bytes[byte_index + 5],
                    bytes[byte_index + 6], bytes[byte_index + 7]
                ]);
            }),
            16 => Ok(quote! {
                let #field_name = #field_type::#from_bytes_method([
                    bytes[byte_index], bytes[byte_index + 1],
                    bytes[byte_index + 2], bytes[byte_index + 3],
                    bytes[byte_index + 4], bytes[byte_index + 5],
                    bytes[byte_index + 6], bytes[byte_index + 7],
                    bytes[byte_index + 8], bytes[byte_index + 9],
                    bytes[byte_index + 10], bytes[byte_index + 11],
                    bytes[byte_index + 12], bytes[byte_index + 13],
                    bytes[byte_index + 14], bytes[byte_index + 15]
                ]);
            }),
            _ => Err(syn::Error::new_spanned(
                field_type,
                "Unsupported primitive type size",
            )),
        }
    }

    /// Create primitive type writing code
    pub fn create_primitive_writing(
        field_name: &Ident,
        field_type: &syn::Type,
        endianness: crate::consts::Endianness,
    ) -> Result<TokenStream, syn::Error> {
        let type_size = crate::utils::get_primitive_type_size(field_type)?;

        // Special handling for char type
        if let syn::Type::Path(tp) = field_type {
            if tp.path.is_ident("char") {
                return match endianness {
                    crate::consts::Endianness::Big => Ok(quote! {
                        let char_bytes = (#field_name as u32).to_be_bytes();
                        bytes.extend_from_slice(&char_bytes);
                        _bit_sum += 32;
                    }),
                    crate::consts::Endianness::Little => Ok(quote! {
                        let char_bytes = (#field_name as u32).to_le_bytes();
                        bytes.extend_from_slice(&char_bytes);
                        _bit_sum += 32;
                    }),
                };
            }
        }

        match endianness {
            crate::consts::Endianness::Big => match type_size {
                1 => Ok(quote! {
                    ::bebytes::BufMut::put_u8(bytes, #field_name as u8);
                    _bit_sum += 8;
                }),
                2 => Ok(quote! {
                    ::bebytes::BufMut::put_u16(bytes, #field_name as u16);
                    _bit_sum += 16;
                }),
                4 => Ok(quote! {
                    ::bebytes::BufMut::put_u32(bytes, #field_name as u32);
                    _bit_sum += 32;
                }),
                8 => Ok(quote! {
                    ::bebytes::BufMut::put_u64(bytes, #field_name as u64);
                    _bit_sum += 64;
                }),
                16 => Ok(quote! {
                    ::bebytes::BufMut::put_u128(bytes, #field_name as u128);
                    _bit_sum += 128;
                }),
                _ => Ok(quote! {
                    let field_slice = &#field_name.to_be_bytes();
                    bytes.extend_from_slice(field_slice);
                    _bit_sum += field_slice.len() * 8;
                }),
            },
            crate::consts::Endianness::Little => match type_size {
                1 => Ok(quote! {
                    ::bebytes::BufMut::put_u8(bytes, #field_name as u8);
                    _bit_sum += 8;
                }),
                2 => Ok(quote! {
                    ::bebytes::BufMut::put_u16_le(bytes, #field_name as u16);
                    _bit_sum += 16;
                }),
                4 => Ok(quote! {
                    ::bebytes::BufMut::put_u32_le(bytes, #field_name as u32);
                    _bit_sum += 32;
                }),
                8 => Ok(quote! {
                    ::bebytes::BufMut::put_u64_le(bytes, #field_name as u64);
                    _bit_sum += 64;
                }),
                16 => Ok(quote! {
                    ::bebytes::BufMut::put_u128_le(bytes, #field_name as u128);
                    _bit_sum += 128;
                }),
                _ => Ok(quote! {
                    let field_slice = &#field_name.to_le_bytes();
                    bytes.extend_from_slice(field_slice);
                    _bit_sum += field_slice.len() * 8;
                }),
            },
        }
    }

    /// Create limit check for bit fields
    pub fn create_bit_field_limit_check(
        field_name: &Ident,
        field_type: &syn::Type,
        size: usize,
    ) -> TokenStream {
        let mask: u128 = (1 << size) - 1;

        // Special handling for char type
        if let syn::Type::Path(tp) = field_type {
            if tp.path.is_ident("char") {
                return quote! {
                    if (#field_name as u32) > #mask as u32 {
                        panic!("Value of field {} is out of range (max value: {})",
                            stringify!(#field_name), #mask);
                    }
                };
            }
        }

        quote! {
            if #field_name > #mask as #field_type {
                panic!("Value of field {} is out of range (max value: {})",
                    stringify!(#field_name), #mask);
            }
        }
    }

    /// Generate field access path from a vector of idents
    pub fn generate_field_access_path(ident_path: &[Ident]) -> TokenStream {
        if ident_path.len() == 1 {
            let ident = &ident_path[0];
            quote!(#ident)
        } else {
            let first = &ident_path[0];
            let rest = &ident_path[1..];
            rest.iter()
                .fold(quote!(#first), |acc, segment| quote!(#acc.#segment))
        }
    }

    /// Calculate bits in byte helper - returns min(8 - `bit_offset`, `total_bits` - `bits_processed`)
    pub fn create_bits_in_byte_calc(
        bit_offset_expr: &TokenStream,
        total_bits: &TokenStream,
        bits_processed: &TokenStream,
    ) -> TokenStream {
        quote! {
            core::cmp::min(8 - #bit_offset_expr, #total_bits - #bits_processed)
        }
    }

    /// Generate aligned multi-byte bit field parsing code
    pub fn create_aligned_multibyte_parsing(
        field_type: &syn::Type,
        from_bytes_method: &TokenStream,
        number_length: usize,
    ) -> TokenStream {
        quote! {
            let slice = &bytes[byte_start..byte_start + #number_length];
            let mut arr = [0u8; #number_length];
            arr.copy_from_slice(slice);
            #field_type::#from_bytes_method(arr)
        }
    }

    /// Generate aligned char bit field parsing code with validation
    pub fn create_aligned_char_parsing(
        from_bytes_method: &TokenStream,
        number_length: usize,
    ) -> TokenStream {
        quote! {
            {
                let slice = &bytes[byte_start..byte_start + #number_length];
                let mut arr = [0u8; #number_length];
                arr.copy_from_slice(slice);
                let char_value = u32::#from_bytes_method(arr);
                char::from_u32(char_value)
                    .ok_or_else(|| ::bebytes::BeBytesError::InvalidDiscriminant {
                        value: (char_value & 0xFF) as u8,
                        type_name: "char",
                    })?
            }
        }
    }

    /// Generate unaligned multi-byte bit field parsing code
    pub fn create_unaligned_multibyte_parsing(
        field_type: &syn::Type,
        size: usize,
        endianness: crate::consts::Endianness,
    ) -> TokenStream {
        let bits_in_byte = create_bits_in_byte_calc(
            &quote!(current_bit_offset),
            &quote!(#size),
            &quote!(bits_read),
        );

        match endianness {
            crate::consts::Endianness::Big => quote! {
                let mut result = 0 as #field_type;
                let mut bits_read = 0;
                let mut byte_idx = byte_start;
                let mut current_bit_offset = bit_offset;

                while bits_read < #size {
                    let bits_in_byte = #bits_in_byte;
                    let byte_val = bytes[byte_idx] as #field_type;
                    let shifted = (byte_val >> (8 - current_bit_offset - bits_in_byte)) & ((1 << bits_in_byte) - 1);
                    result = (result << bits_in_byte) | shifted;

                    bits_read += bits_in_byte;
                    byte_idx += 1;
                    current_bit_offset = 0;
                }
                result
            },
            crate::consts::Endianness::Little => quote! {
                let mut result = 0 as #field_type;
                let mut bits_read = 0;
                let mut byte_idx = byte_start;
                let mut current_bit_offset = bit_offset;

                while bits_read < #size {
                    let bits_in_byte = #bits_in_byte;
                    let byte_val = bytes[byte_idx] as #field_type;
                    let shifted = (byte_val >> current_bit_offset) & ((1 << bits_in_byte) - 1);
                    result |= shifted << bits_read;

                    bits_read += bits_in_byte;
                    byte_idx += 1;
                    current_bit_offset = 0;
                }
                result
            },
        }
    }

    /// Generate unaligned char bit field parsing code with validation
    pub fn create_unaligned_char_parsing(
        size: usize,
        endianness: crate::consts::Endianness,
    ) -> TokenStream {
        let bits_in_byte = create_bits_in_byte_calc(
            &quote!(current_bit_offset),
            &quote!(#size),
            &quote!(bits_read),
        );

        match endianness {
            crate::consts::Endianness::Big => quote! {
                {
                    let mut result = 0u32;
                    let mut bits_read = 0;
                    let mut byte_idx = byte_start;
                    let mut current_bit_offset = bit_offset;

                    while bits_read < #size {
                        let bits_in_byte = #bits_in_byte;
                        let byte_val = bytes[byte_idx] as u32;
                        let shifted = (byte_val >> (8 - current_bit_offset - bits_in_byte)) & ((1 << bits_in_byte) - 1);
                        result = (result << bits_in_byte) | shifted;

                        bits_read += bits_in_byte;
                        byte_idx += 1;
                        current_bit_offset = 0;
                    }
                    char::from_u32(result)
                        .ok_or_else(|| ::bebytes::BeBytesError::InvalidDiscriminant {
                            value: (result & 0xFF) as u8,
                            type_name: "char",
                        })?
                }
            },
            crate::consts::Endianness::Little => quote! {
                {
                    let mut result = 0u32;
                    let mut bits_read = 0;
                    let mut byte_idx = byte_start;
                    let mut current_bit_offset = bit_offset;

                    while bits_read < #size {
                        let bits_in_byte = #bits_in_byte;
                        let byte_val = bytes[byte_idx] as u32;
                        let shifted = (byte_val >> current_bit_offset) & ((1 << bits_in_byte) - 1);
                        result |= shifted << bits_read;

                        bits_read += bits_in_byte;
                        byte_idx += 1;
                        current_bit_offset = 0;
                    }
                    char::from_u32(result)
                        .ok_or_else(|| ::bebytes::BeBytesError::InvalidDiscriminant {
                            value: (result & 0xFF) as u8,
                            type_name: "char",
                        })?
                }
            },
        }
    }

    /// Generate aligned multi-byte bit field writing code
    pub fn create_aligned_multibyte_writing(
        field_type: &syn::Type,
        to_bytes_method: &TokenStream,
        number_length: usize,
    ) -> TokenStream {
        quote! {
            let value_bytes = #field_type::#to_bytes_method(value);
            if bytes.len() < byte_start + #number_length {
                bytes.resize(byte_start + #number_length, 0);
            }
            bytes[byte_start..byte_start + #number_length].copy_from_slice(&value_bytes);
        }
    }

    /// Generate unaligned multi-byte bit field writing code
    pub fn create_unaligned_multibyte_writing(
        field_type: &syn::Type,
        size: usize,
        endianness: crate::consts::Endianness,
    ) -> TokenStream {
        let bits_in_byte = create_bits_in_byte_calc(
            &quote!(current_bit_offset),
            &quote!(#size),
            &quote!(bits_written),
        );

        match endianness {
            crate::consts::Endianness::Big => quote! {
                let mut remaining_value = value;
                let mut bits_written = 0;
                let mut byte_idx = byte_start;
                let mut current_bit_offset = bit_offset;

                while bits_written < #size {
                    let bits_in_byte = #bits_in_byte;
                    let mask = ((1 << bits_in_byte) - 1) as u8;
                    let shift = #size - bits_written - bits_in_byte;
                    let byte_bits = ((remaining_value >> shift) & mask as #field_type) as u8;

                    if bytes.len() <= byte_idx {
                        bytes.resize(byte_idx + 1, 0);
                    }
                    bytes[byte_idx] |= byte_bits << (8 - current_bit_offset - bits_in_byte);

                    bits_written += bits_in_byte;
                    byte_idx += 1;
                    current_bit_offset = 0;
                }
            },
            crate::consts::Endianness::Little => quote! {
                let mut remaining_value = value;
                let mut bits_written = 0;
                let mut byte_idx = byte_start;
                let mut current_bit_offset = bit_offset;

                while bits_written < #size {
                    let bits_in_byte = #bits_in_byte;
                    let mask = ((1 << bits_in_byte) - 1) as #field_type;
                    let byte_bits = (remaining_value & mask) as u8;

                    if bytes.len() <= byte_idx {
                        bytes.resize(byte_idx + 1, 0);
                    }
                    bytes[byte_idx] |= byte_bits << current_bit_offset;

                    remaining_value >>= bits_in_byte;
                    bits_written += bits_in_byte;
                    byte_idx += 1;
                    current_bit_offset = 0;
                }
            },
        }
    }

    /// Generate aligned char bit field writing code
    pub fn create_aligned_char_writing(
        to_bytes_method: &TokenStream,
        number_length: usize,
    ) -> TokenStream {
        quote! {
            let value_bytes = u32::#to_bytes_method(value as u32);
            if bytes.len() < byte_start + #number_length {
                bytes.resize(byte_start + #number_length, 0);
            }
            bytes[byte_start..byte_start + #number_length].copy_from_slice(&value_bytes);
        }
    }

    /// Generate unaligned char bit field writing code
    pub fn create_unaligned_char_writing(
        size: usize,
        endianness: crate::consts::Endianness,
    ) -> TokenStream {
        let bits_in_byte = create_bits_in_byte_calc(
            &quote!(current_bit_offset),
            &quote!(#size),
            &quote!(bits_written),
        );

        match endianness {
            crate::consts::Endianness::Big => quote! {
                let mut remaining_value = value as u32;
                let mut bits_written = 0;
                let mut byte_idx = byte_start;
                let mut current_bit_offset = bit_offset;

                while bits_written < #size {
                    let bits_in_byte = #bits_in_byte;
                    let mask = ((1 << bits_in_byte) - 1) as u8;
                    let shift = #size - bits_written - bits_in_byte;
                    let byte_bits = ((remaining_value >> shift) & mask as u32) as u8;

                    if bytes.len() <= byte_idx {
                        bytes.resize(byte_idx + 1, 0);
                    }
                    bytes[byte_idx] |= byte_bits << (8 - current_bit_offset - bits_in_byte);

                    bits_written += bits_in_byte;
                    byte_idx += 1;
                    current_bit_offset = 0;
                }
            },
            crate::consts::Endianness::Little => quote! {
                let mut remaining_value = value as u32;
                let mut bits_written = 0;
                let mut byte_idx = byte_start;
                let mut current_bit_offset = bit_offset;

                while bits_written < #size {
                    let bits_in_byte = #bits_in_byte;
                    let mask = ((1 << bits_in_byte) - 1) as u32;
                    let byte_bits = (remaining_value & mask) as u8;

                    if bytes.len() <= byte_idx {
                        bytes.resize(byte_idx + 1, 0);
                    }
                    bytes[byte_idx] |= byte_bits << current_bit_offset;

                    remaining_value >>= bits_in_byte;
                    bits_written += bits_in_byte;
                    byte_idx += 1;
                    current_bit_offset = 0;
                }
            },
        }
    }

    /// Generate single-byte bit field parsing code based on endianness
    pub fn create_single_byte_bit_parsing(
        field_name: &Ident,
        field_type: &syn::Type,
        size: usize,
        mask: u128,
        endianness: crate::consts::Endianness,
    ) -> TokenStream {
        match endianness {
            crate::consts::Endianness::Big => quote! {
                let #field_name = {
                    let byte_idx = _bit_sum / 8;
                    let bit_offset = _bit_sum % 8;

                    // Check if field spans two bytes
                    if bit_offset + #size > 8 {
                        // Field spans two bytes
                        let mut result = 0 as #field_type;
                        let mut bits_read = 0;
                        let mut current_byte_idx = byte_idx;
                        let mut current_bit_offset = bit_offset;

                        while bits_read < #size {
                            if current_byte_idx >= bytes.len() {
                                return Err(::bebytes::BeBytesError::InsufficientData {
                                    expected: current_byte_idx + 1,
                                    actual: bytes.len(),
                                });
                            }

                            let bits_in_byte = core::cmp::min(8 - current_bit_offset, #size - bits_read);
                            let byte_val = bytes[current_byte_idx] as #field_type;
                            let shifted = (byte_val >> (8 - current_bit_offset - bits_in_byte)) & ((1 << bits_in_byte) - 1);
                            result = (result << bits_in_byte) | shifted;

                            bits_read += bits_in_byte;
                            current_byte_idx += 1;
                            current_bit_offset = 0;
                        }
                        result
                    } else {
                        // Field fits in single byte
                        let mask = #mask as #field_type;
                        let byte_val = bytes[byte_idx] as #field_type;
                        (byte_val >> (8 - bit_offset - #size)) & mask
                    }
                };
                _bit_sum += #size;
            },
            crate::consts::Endianness::Little => quote! {
                let #field_name = {
                    let byte_idx = _bit_sum / 8;
                    let bit_offset = _bit_sum % 8;
                    let mask = #mask as #field_type;
                    let byte_val = bytes[byte_idx] as #field_type;
                    (byte_val >> bit_offset) & mask
                };
                _bit_sum += #size;
            },
        }
    }

    /// Generate single-byte bit field writing code based on endianness
    pub fn create_single_byte_bit_writing(
        field_name: &Ident,
        size: usize,
        mask: u128,
        endianness: crate::consts::Endianness,
    ) -> TokenStream {
        match endianness {
            crate::consts::Endianness::Big => quote! {
                {
                    let byte_idx = _bit_sum / 8;
                    let bit_offset = _bit_sum % 8;

                    // Check if field spans two bytes
                    if bit_offset + #size > 8 {
                        // Field spans two bytes - use multi-byte approach
                        let mut remaining_value = #field_name as u8;
                        let mut bits_written = 0;
                        let mut current_byte_idx = byte_idx;
                        let mut current_bit_offset = bit_offset;

                        while bits_written < #size {
                            let bits_in_byte = core::cmp::min(8 - current_bit_offset, #size - bits_written);
                            let bit_mask = ((1 << bits_in_byte) - 1) as u8;
                            let shift = #size - bits_written - bits_in_byte;
                            let byte_bits = ((remaining_value >> shift) & bit_mask) as u8;

                            if bytes.len() <= current_byte_idx {
                                bytes.resize(current_byte_idx + 1, 0);
                            }
                            bytes[current_byte_idx] |= byte_bits << (8 - current_bit_offset - bits_in_byte);

                            bits_written += bits_in_byte;
                            current_byte_idx += 1;
                            current_bit_offset = 0;
                        }
                    } else {
                        // Field fits in single byte
                        let mask = #mask as u8;
                        if bytes.len() <= byte_idx {
                            bytes.resize(byte_idx + 1, 0);
                        }
                        bytes[byte_idx] |= ((#field_name as u8) & mask) << (8 - bit_offset - #size);
                    }
                }
                _bit_sum += #size;
            },
            crate::consts::Endianness::Little => quote! {
                {
                    let byte_idx = _bit_sum / 8;
                    let bit_offset = _bit_sum % 8;

                    // Check if field spans multiple bytes
                    if bit_offset + #size > 8 {
                        // Field spans multiple bytes - use multi-byte approach for LE
                        let mut remaining_value = #field_name as u16; // Use larger type for multi-byte
                        let mut bits_written = 0;
                        let mut current_byte_idx = byte_idx;
                        let mut current_bit_offset = bit_offset;

                        while bits_written < #size {
                            let bits_in_byte = core::cmp::min(8 - current_bit_offset, #size - bits_written);
                            let bit_mask = ((1 << bits_in_byte) - 1) as u16;
                            let byte_bits = (remaining_value & bit_mask) as u8;

                            if bytes.len() <= current_byte_idx {
                                bytes.resize(current_byte_idx + 1, 0);
                            }
                            bytes[current_byte_idx] |= byte_bits << current_bit_offset;

                            remaining_value >>= bits_in_byte;
                            bits_written += bits_in_byte;
                            current_byte_idx += 1;
                            current_bit_offset = 0;
                        }
                    } else {
                        // Field fits in single byte
                        let mask = #mask as u8;
                        if bytes.len() <= byte_idx {
                            bytes.resize(byte_idx + 1, 0);
                        }
                        bytes[byte_idx] |= ((#field_name as u8) & mask) << bit_offset;
                    }
                }
                _bit_sum += #size;
            },
        }
    }
}

/// Functional attribute parsing
pub mod functional_attrs {
    use super::{error_utils, AttributeData, ParseResult, Vec};
    use syn::{parse::Parser, LitInt};

    /// Parse attributes functionally without side effects
    pub fn parse_attributes_functional(
        attributes: &[syn::Attribute],
    ) -> ParseResult<AttributeData> {
        let results: Vec<Result<Option<AttributeData>, syn::Error>> = attributes
            .iter()
            .map(|attr| {
                if attr.path().is_ident("bits") {
                    parse_bits_attribute_functional(attr).map(|size_opt| {
                        let mut data = AttributeData::new().with_bits_attribute();
                        if let Some(size) = size_opt {
                            data = data.with_size(size);
                        }
                        Some(data)
                    })
                } else if attr.path().is_ident("With") {
                    parse_with_attribute_with_expressions(attr)
                } else if attr.path().is_ident("FromField") {
                    parse_from_field_attribute_functional(attr)
                        .map(|field| Some(AttributeData::new().with_field(field)))
                } else if attr.path().is_ident("size") {
                    parse_size_attribute_with_expressions(attr)
                } else {
                    Ok(None)
                }
            })
            .collect();

        let flattened: Vec<Result<AttributeData, syn::Error>> =
            results.into_iter().filter_map(Result::transpose).collect();

        error_utils::aggregate_results(flattened.into_iter()).map(AttributeData::merge)
    }

    /// Parse bits attribute functionally
    pub fn parse_bits_attribute_functional(
        attr: &syn::Attribute,
    ) -> Result<Option<usize>, syn::Error> {
        // Check the meta type first
        match &attr.meta {
            syn::Meta::Path(_) => {
                // #[bits] without parentheses - not allowed by Rust for derive macro attributes
                // This case won't actually be reached due to Rust's validation
                Ok(None)
            }
            syn::Meta::List(list) => {
                // Check if tokens are empty first
                if list.tokens.is_empty() {
                    // #[bits()] with empty parentheses - auto-size
                    return Ok(None);
                }

                // Try to parse as integer literal
                let parser =
                    syn::punctuated::Punctuated::<LitInt, syn::Token![,]>::parse_terminated;
                let parsed = parser.parse2(list.tokens.clone())?;

                if let Some(first) = parsed.first() {
                    // #[bits(N)] with explicit size
                    Ok(Some(first.base10_parse()?))
                } else {
                    Err(syn::Error::new_spanned(
                        attr,
                        "Expected integer literal in #[bits(N)]",
                    ))
                }
            }
            syn::Meta::NameValue(_) => Err(syn::Error::new_spanned(
                attr,
                "Expected #[bits(N)] or #[bits()], not name-value style",
            )),
        }
    }

    /// Parse with attribute with support for size expressions
    pub fn parse_with_attribute_with_expressions(
        attr: &syn::Attribute,
    ) -> Result<Option<AttributeData>, syn::Error> {
        let mut size = None;
        let mut size_expression = None;

        // Parse the content inside With(...)
        match &attr.meta {
            syn::Meta::List(list) => {
                // Parse the tokens inside With(...)
                let tokens = &list.tokens;

                // Try to parse as "size(...)" pattern
                let tokens_str = tokens.to_string();
                if tokens_str.starts_with("size") {
                    // Find the content inside size(...)
                    if let Some(start) = tokens_str.find('(') {
                        if let Some(end) = tokens_str.rfind(')') {
                            let expr_content = &tokens_str[start + 1..end];

                            // Try to parse as integer literal first
                            if let Ok(n) = expr_content.trim().parse::<usize>() {
                                size = Some(n);
                            } else {
                                // Parse as expression
                                let parsed_expr =
                                    crate::size_expr::SizeExpression::parse(expr_content.trim())?;
                                size_expression = Some(parsed_expr);
                            }
                        } else {
                            return Err(syn::Error::new_spanned(
                                attr,
                                "Missing closing parenthesis in size attribute",
                            ));
                        }
                    } else {
                        return Err(syn::Error::new_spanned(attr, "Expected size(...) format"));
                    }
                } else {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "Only size attribute is supported in With",
                    ));
                }
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    attr,
                    "With attribute must have parentheses: #[With(size(...))]",
                ));
            }
        }

        if size.is_some() || size_expression.is_some() {
            let mut attr_data = AttributeData::new();
            if let Some(s) = size {
                attr_data = attr_data.with_size(s);
            }
            if let Some(expr) = size_expression {
                attr_data = attr_data.with_size_expression(expr);
            }
            Ok(Some(attr_data))
        } else {
            Ok(None)
        }
    }

    /// Parse standalone size attribute with expression support
    pub fn parse_size_attribute_with_expressions(
        attr: &syn::Attribute,
    ) -> Result<Option<AttributeData>, syn::Error> {
        // Parse the content inside #[size(...)]
        match &attr.meta {
            syn::Meta::List(list) => {
                let tokens = &list.tokens;
                let expr_str = tokens.to_string();
                let parsed_expr = crate::size_expr::SizeExpression::parse(&expr_str)?;
                let attr_data = AttributeData::new().with_size_expression(parsed_expr);
                Ok(Some(attr_data))
            }
            _ => Err(syn::Error::new_spanned(
                attr,
                "size attribute must have parentheses: #[size(expression)]",
            )),
        }
    }

    /// Parse from field attribute functionally
    pub fn parse_from_field_attribute_functional(
        attr: &syn::Attribute,
    ) -> Result<Vec<syn::Ident>, syn::Error> {
        let field_path: Vec<syn::Ident>;

        // Parse the attribute content as a token stream
        match &attr.meta {
            syn::Meta::List(list) => {
                // Parse tokens inside FromField(...)
                let tokens = list.tokens.clone();
                let parser = syn::punctuated::Punctuated::<syn::Ident, syn::Token![.]>::parse_separated_nonempty;
                match parser.parse2(tokens) {
                    Ok(punctuated) => {
                        // Convert punctuated list to Vec<Ident>
                        field_path = punctuated.into_iter().collect();
                    }
                    Err(e) => return Err(e),
                }
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    attr,
                    "Expected #[FromField(field_name)] or #[FromField(header.qdcount)]",
                ))
            }
        }

        if field_path.is_empty() {
            Err(syn::Error::new_spanned(attr, "Missing field name or path"))
        } else {
            Ok(field_path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use quote::quote;

    #[test]
    fn test_processing_context_builder() {
        let ctx = ProcessingContext::new(crate::consts::Endianness::Big)
            .with_bit_position(16)
            .with_last_field(true);

        assert_eq!(ctx.bit_position, 16);
        assert!(ctx.is_last_field);
    }

    #[test]
    fn test_field_data_builder() {
        let result = FieldProcessResult::new(
            quote! { check },
            quote! { parse },
            quote! { write },
            quote! { direct_write },
            quote! { access },
            quote! { bit_sum },
        );

        let builder = FieldDataBuilder::new();
        let field_data = builder.add_result(result).build();

        assert_eq!(field_data.field_limit_check.len(), 1);
        assert_eq!(field_data.field_parsing.len(), 1);
        assert_eq!(field_data.field_writing.len(), 1);
        assert_eq!(field_data.named_fields.len(), 1);
        assert_eq!(field_data.bit_sum.len(), 1);
    }

    #[test]
    fn test_attribute_data_merge() {
        let attr1 = AttributeData::new().with_size(8);
        let attr2 = AttributeData::new().with_bits_attribute();
        let attr3 = AttributeData::new().with_field(vec![Ident::new("test", Span::call_site())]);

        let merged = AttributeData::merge(vec![attr1, attr2, attr3]);

        assert_eq!(merged.size, Some(8));
        assert!(merged.is_bits_attribute);
        assert!(merged.field.is_some());
    }

    #[test]
    fn test_error_aggregation() {
        let results = vec![
            Ok(1),
            Err(SynError::new(Span::call_site(), "error1")),
            Ok(2),
            Err(SynError::new(Span::call_site(), "error2")),
        ];

        let result = error_utils::aggregate_results(results.into_iter());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().len(), 2);
    }

    #[test]
    fn test_successful_aggregation() {
        let results = vec![Ok(1), Ok(2), Ok(3)];
        let result = error_utils::aggregate_results(results.into_iter());
        assert_eq!(result.unwrap(), vec![1, 2, 3]);
    }
}
