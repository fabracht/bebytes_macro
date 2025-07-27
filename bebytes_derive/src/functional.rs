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
    pub accessor: TokenStream,
    pub bit_sum: TokenStream,
}

impl FieldProcessResult {
    pub fn new(
        limit_check: TokenStream,
        parsing: TokenStream,
        writing: TokenStream,
        accessor: TokenStream,
        bit_sum: TokenStream,
    ) -> Self {
        Self {
            limit_check,
            parsing,
            writing,
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
    accessors: Vec<TokenStream>,
    bit_sums: Vec<TokenStream>,
}

impl FieldDataBuilder {
    pub fn new() -> Self {
        Self {
            limit_checks: Vec::new(),
            parsings: Vec::new(),
            writings: Vec::new(),
            accessors: Vec::new(),
            bit_sums: Vec::new(),
        }
    }

    pub fn add_result(mut self, result: FieldProcessResult) -> Self {
        self.limit_checks.push(result.limit_check);
        self.parsings.push(result.parsing);
        self.writings.push(result.writing);
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

    /// Merge multiple `AttributeData` instances, prioritizing non-`None` values
    pub fn merge(attrs: Vec<Self>) -> Self {
        attrs.into_iter().fold(Self::default(), |mut acc, attr| {
            acc.size = attr.size.or(acc.size);
            acc.field = attr.field.or(acc.field);
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
            quote! { let #field_name = self.#field_name.to_owned(); }
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
                _bit_sum = ((_bit_sum + 7) / 8) * 8;
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

        match endianness {
            crate::consts::Endianness::Big => match type_size {
                1 => Ok(quote! {
                    let #field_name = bytes[byte_index] as #field_type;
                }),
                2 => Ok(quote! {
                    let #field_name = #field_type::from_be_bytes([
                        bytes[byte_index], bytes[byte_index + 1]
                    ]);
                }),
                4 => Ok(quote! {
                    let #field_name = #field_type::from_be_bytes([
                        bytes[byte_index], bytes[byte_index + 1],
                        bytes[byte_index + 2], bytes[byte_index + 3]
                    ]);
                }),
                8 => Ok(quote! {
                    let #field_name = #field_type::from_be_bytes([
                        bytes[byte_index], bytes[byte_index + 1],
                        bytes[byte_index + 2], bytes[byte_index + 3],
                        bytes[byte_index + 4], bytes[byte_index + 5],
                        bytes[byte_index + 6], bytes[byte_index + 7]
                    ]);
                }),
                16 => Ok(quote! {
                    let #field_name = #field_type::from_be_bytes([
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
            },
            crate::consts::Endianness::Little => match type_size {
                1 => Ok(quote! {
                    let #field_name = bytes[byte_index] as #field_type;
                }),
                2 => Ok(quote! {
                    let #field_name = #field_type::from_le_bytes([
                        bytes[byte_index], bytes[byte_index + 1]
                    ]);
                }),
                4 => Ok(quote! {
                    let #field_name = #field_type::from_le_bytes([
                        bytes[byte_index], bytes[byte_index + 1],
                        bytes[byte_index + 2], bytes[byte_index + 3]
                    ]);
                }),
                8 => Ok(quote! {
                    let #field_name = #field_type::from_le_bytes([
                        bytes[byte_index], bytes[byte_index + 1],
                        bytes[byte_index + 2], bytes[byte_index + 3],
                        bytes[byte_index + 4], bytes[byte_index + 5],
                        bytes[byte_index + 6], bytes[byte_index + 7]
                    ]);
                }),
                16 => Ok(quote! {
                    let #field_name = #field_type::from_le_bytes([
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
            },
        }
    }

    /// Create primitive type writing code
    pub fn create_primitive_writing(
        field_name: &Ident,
        field_type: &syn::Type,
        endianness: crate::consts::Endianness,
    ) -> Result<TokenStream, syn::Error> {
        let type_size = crate::utils::get_primitive_type_size(field_type)?;

        match endianness {
            crate::consts::Endianness::Big => match type_size {
                1 => Ok(quote! {
                    bytes.push(#field_name as u8);
                    _bit_sum += 8;
                }),
                _ => Ok(quote! {
                    let field_slice = &#field_name.to_be_bytes();
                    bytes.extend_from_slice(field_slice);
                    _bit_sum += field_slice.len() * 8;
                }),
            },
            crate::consts::Endianness::Little => match type_size {
                1 => Ok(quote! {
                    bytes.push(#field_name as u8);
                    _bit_sum += 8;
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

    /// Generate unaligned multi-byte bit field parsing code
    pub fn create_unaligned_multibyte_parsing(field_type: &syn::Type, size: usize) -> TokenStream {
        let bits_in_byte = create_bits_in_byte_calc(
            &quote!(current_bit_offset),
            &quote!(#size),
            &quote!(bits_read),
        );

        quote! {
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
    pub fn create_unaligned_multibyte_writing(field_type: &syn::Type, size: usize) -> TokenStream {
        let bits_in_byte = create_bits_in_byte_calc(
            &quote!(current_bit_offset),
            &quote!(#size),
            &quote!(bits_written),
        );

        quote! {
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
                    let mask = #mask as u8;
                    if bytes.len() <= byte_idx {
                        bytes.resize(byte_idx + 1, 0);
                    }
                    bytes[byte_idx] |= ((#field_name as u8) & mask) << bit_offset;
                }
                _bit_sum += #size;
            },
        }
    }

    /// Generate auto-bits single byte parsing (for enums with `__BEBYTES_MIN_BITS`)
    pub fn create_auto_bits_single_byte_parsing(
        field_type: &syn::Type,
        endianness: crate::consts::Endianness,
    ) -> TokenStream {
        let shift = match endianness {
            crate::consts::Endianness::Big => quote! { 8 - bit_offset - SIZE },
            crate::consts::Endianness::Little => quote! { bit_offset },
        };

        quote! {
            {
                if byte_idx >= bytes.len() {
                    return Err(::bebytes::BeBytesError::InsufficientData {
                        expected: byte_idx + 1,
                        actual: bytes.len(),
                    });
                }

                let bits = {
                    const MASK: u8 = (1 << SIZE) - 1;
                    let byte_val = bytes[byte_idx];
                    (byte_val >> (#shift)) & MASK
                };

                // Convert from discriminant value to enum
                #field_type::try_from(bits)?
            }
        }
    }

    /// Generate auto-bits two byte parsing (for enums spanning two bytes)
    pub fn create_auto_bits_two_byte_parsing(
        field_type: &syn::Type,
        endianness: crate::consts::Endianness,
    ) -> TokenStream {
        let (first_bits_expr, second_bits_expr, combine_expr) = match endianness {
            crate::consts::Endianness::Big => (
                quote! { first_byte & ((1 << bits_from_first) - 1) },
                quote! { second_byte >> (8 - bits_from_second) },
                quote! { (first_bits << bits_from_second) | second_bits },
            ),
            crate::consts::Endianness::Little => (
                quote! { (first_byte >> bit_offset) & ((1 << bits_from_first) - 1) },
                quote! { second_byte & ((1 << bits_from_second) - 1) },
                quote! { first_bits | (second_bits << bits_from_first) },
            ),
        };

        quote! {
            {
                let bits_from_first = 8 - bit_offset;
                let bits_from_second = SIZE - bits_from_first;

                if byte_idx + 1 >= bytes.len() {
                    return Err(::bebytes::BeBytesError::InsufficientData {
                        expected: byte_idx + 1,
                        actual: bytes.len(),
                    });
                }

                let first_byte = bytes[byte_idx];
                let second_byte = bytes[byte_idx + 1];

                let first_bits = #first_bits_expr;
                let second_bits = #second_bits_expr;

                let bits = #combine_expr;

                // Convert from discriminant value to enum
                #field_type::try_from(bits)?
            }
        }
    }

    /// Generate auto-bits single byte writing (for enums with `__BEBYTES_MIN_BITS`)
    pub fn create_auto_bits_single_byte_writing(
        field_name: &Ident,
        size_const: &TokenStream,
        endianness: crate::consts::Endianness,
    ) -> TokenStream {
        let shift = match endianness {
            crate::consts::Endianness::Big => quote! { 8 - bit_offset - SIZE },
            crate::consts::Endianness::Little => quote! { bit_offset },
        };

        quote! {
            // Convert enum to its discriminant value
            let bits = #field_name as u8;

            const MASK: u8 = (1 << SIZE) - 1;

            if bytes.len() <= byte_idx {
                bytes.resize(byte_idx + 1, 0);
            }
            bytes[byte_idx] |= (bits & MASK) << (#shift);
            _bit_sum += #size_const;
        }
    }

    /// Generate auto-bits two byte writing (for enums spanning two bytes)
    pub fn create_auto_bits_two_byte_writing(
        field_name: &Ident,
        size_const: &TokenStream,
        endianness: crate::consts::Endianness,
    ) -> TokenStream {
        let (first_byte_expr, second_byte_expr) = match endianness {
            crate::consts::Endianness::Big => (
                quote! { (bits >> bits_in_second) & first_mask },
                quote! { (bits & second_mask) << (8 - bits_in_second) },
            ),
            crate::consts::Endianness::Little => (
                quote! { (bits & first_mask) << bit_offset },
                quote! { (bits >> bits_in_first) & second_mask },
            ),
        };

        quote! {
            // Convert enum to its discriminant value
            let bits = #field_name as u8;

            let bits_in_first = 8 - bit_offset;
            let bits_in_second = SIZE - bits_in_first;

            if bytes.len() <= byte_idx + 1 {
                bytes.resize(byte_idx + 2, 0);
            }

            // Write to first byte (lower bits of the value)
            let first_mask = (1 << bits_in_first) - 1;
            bytes[byte_idx] |= #first_byte_expr;

            // Write to second byte (upper bits of the value)
            let second_mask = (1 << bits_in_second) - 1;
            bytes[byte_idx + 1] |= #second_byte_expr;
            _bit_sum += #size_const;
        }
    }
}

/// Functional attribute parsing
pub mod functional_attrs {
    use super::{error_utils, AttributeData, ParseResult, Vec};
    use syn::{parenthesized, parse::Parser, LitInt};

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
                    parse_with_attribute_functional(attr)
                        .map(|size| size.map(|s| AttributeData::new().with_size(s)))
                } else if attr.path().is_ident("FromField") {
                    parse_from_field_attribute_functional(attr)
                        .map(|field| Some(AttributeData::new().with_field(field)))
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

    /// Parse with attribute functionally
    pub fn parse_with_attribute_functional(
        attr: &syn::Attribute,
    ) -> Result<Option<usize>, syn::Error> {
        let mut size = None;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("size") {
                let content;
                parenthesized!(content in meta.input);
                let lit: LitInt = content.parse()?;
                let n: usize = lit.base10_parse()?;
                size = Some(n);
                Ok(())
            } else {
                Err(meta.error("Allowed attributes are `size` - Example: #[With(size(3))]"))
            }
        })?;
        Ok(size)
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
