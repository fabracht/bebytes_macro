use crate::{attrs, utils};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

enum FieldType {
    BitsField(usize), // only size, position is auto-calculated
    PrimitiveType,
    Array(usize),                                   // array_length
    Vector(Option<usize>, Option<Vec<syn::Ident>>), // size, vec_size_ident
    VarString,                                      // Variable-length string with prefix
    OptionType,
    CustomType,
}

struct FieldContext<'a> {
    field: &'a syn::Field,
    field_name: syn::Ident,
    field_type: &'a syn::Type,
    is_last_field: bool,
}

pub struct FieldData {
    pub field_limit_check: Vec<proc_macro2::TokenStream>,
    pub errors: Vec<proc_macro2::TokenStream>,
    pub field_parsing: Vec<proc_macro2::TokenStream>,
    pub bit_sum: Vec<proc_macro2::TokenStream>,
    pub field_writing: Vec<proc_macro2::TokenStream>,
    pub named_fields: Vec<proc_macro2::TokenStream>,
    pub total_size: usize,
}

pub struct StructContext<'a> {
    pub field_limit_check: &'a mut Vec<proc_macro2::TokenStream>,
    pub errors: &'a mut Vec<proc_macro2::TokenStream>,
    pub field_parsing: &'a mut Vec<proc_macro2::TokenStream>,
    pub bit_sum: &'a mut Vec<proc_macro2::TokenStream>,
    pub field_writing: &'a mut Vec<proc_macro2::TokenStream>,
    pub named_fields: &'a mut Vec<proc_macro2::TokenStream>,
    pub fields: &'a syn::FieldsNamed,
    pub endianness: crate::consts::Endianness,
}

fn determine_field_type(
    context: &FieldContext,
    attrs: &[syn::Attribute],
    errors: &mut Vec<proc_macro2::TokenStream>,
) -> Option<FieldType> {
    let mut bits_attribute_present = false;
    let (size, vec_size_ident) =
        attrs::parse_attributes(attrs, &mut bits_attribute_present, errors);

    if bits_attribute_present {
        if let Some(size) = size {
            // Explicit size: #[bits(N)]
            if let syn::Type::Path(tp) = context.field_type {
                if !utils::is_supported_primitive_type(tp) {
                    let error = syn::Error::new(
                        context.field_type.span(),
                        "Unsupported type for bits attribute. Only integer types (u8, i8, u16, i16, u32, i32, u64, i64, u128, i128) and char are supported",
                    );
                    errors.push(error.to_compile_error());
                    return None;
                }
            }
            return Some(FieldType::BitsField(size));
        }
        // Empty #[bits()] is no longer supported
        let error = syn::Error::new(
            context.field.span(),
            "Size required for bits attribute. Use #[bits(N)] where N is the number of bits needed for your field",
        );
        errors.push(error.to_compile_error());
        return None;
    }

    match context.field_type {
        syn::Type::Path(tp) if utils::is_primitive_type(tp) => Some(FieldType::PrimitiveType),
        syn::Type::Array(arr) => {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(len),
                ..
            }) = &arr.len
            {
                if let Ok(length) = len.base10_parse() {
                    return Some(FieldType::Array(length));
                }
            }
            None
        }
        syn::Type::Path(tp) if !tp.path.segments.is_empty() => {
            let segment = &tp.path.segments[0];
            match &segment.ident {
                ident if ident == "Vec" => Some(FieldType::Vector(size, vec_size_ident)),
                ident if ident == "Option" => Some(FieldType::OptionType),
                ident if ident == "VarString" || ident == "VarString8" || ident == "VarString16" || ident == "VarString32" => Some(FieldType::VarString),
                ident if !utils::is_primitive_identity(ident) => Some(FieldType::CustomType),
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn handle_struct(context: &mut StructContext) {
    // First validate byte completeness
    if let Err(validation_error) = crate::bit_validation::validate_byte_completeness(context.fields)
    {
        context.errors.push(validation_error);
        return;
    }

    // Initialize ProcessingContext for functional approach
    let processing_ctx = crate::functional::ProcessingContext::new(context.endianness);

    // Use FieldDataBuilder for functional accumulation
    let mut builder = crate::functional::FieldDataBuilder::new();
    let mut errors = Vec::new();

    // Track current bit position for auto-calculation
    let mut current_bit_position = 0;

    for (idx, field) in context.fields.named.iter().enumerate() {
        let is_last = idx == context.fields.named.len() - 1;

        let field_context = FieldContext {
            field,
            field_name: field.ident.clone().unwrap(),
            field_type: &field.ty,
            is_last_field: is_last,
        };

        // Create a new processing context for this field
        let field_processing_ctx = processing_ctx
            .clone()
            .with_bit_position(current_bit_position)
            .with_last_field(is_last);

        if let Some(field_type) = determine_field_type(&field_context, &field.attrs, &mut errors) {
            let result = process_field_type(
                &field_context,
                field_type,
                &field_processing_ctx,
                &mut current_bit_position,
            );

            match result {
                Ok(field_result) => {
                    builder = builder.add_result(field_result);
                }
                Err(e) => errors.push(e.to_compile_error()),
            }
        }
    }

    // Build the final FieldData
    let mut field_data = builder.build();
    field_data.errors = errors;
    field_data.total_size = current_bit_position / 8;

    context
        .field_limit_check
        .extend(field_data.field_limit_check);
    context.errors.extend(field_data.errors);
    context.field_parsing.extend(field_data.field_parsing);
    context.bit_sum.extend(field_data.bit_sum);
    context.field_writing.extend(field_data.field_writing);
    context.named_fields.extend(field_data.named_fields);
}

// New functional field processor
fn process_field_type(
    context: &FieldContext,
    field_type: FieldType,
    processing_ctx: &crate::functional::ProcessingContext,
    current_bit_position: &mut usize,
) -> Result<crate::functional::FieldProcessResult, syn::Error> {
    match field_type {
        FieldType::BitsField(size) => {
            let result = process_bits_field_functional(
                context,
                size,
                processing_ctx,
                *current_bit_position,
            )?;
            *current_bit_position += size;
            Ok(result)
        }
        FieldType::PrimitiveType => {
            let result = process_primitive_type_functional(context, processing_ctx)?;
            // Update bit position based on primitive size
            let field_size = utils::get_primitive_type_size(context.field_type)?;
            *current_bit_position += field_size * 8;
            Ok(result)
        }
        FieldType::Array(length) => {
            let result = process_array_functional(context, length, processing_ctx)?;
            // Arrays are always byte arrays in our case
            *current_bit_position += length * 8;
            Ok(result)
        }
        FieldType::Vector(size, vec_size_ident) => {
            let result = process_vector_functional(context, size, vec_size_ident, processing_ctx)?;
            // Vectors have variable size, but we need to track something for bit field positioning
            if let Some(s) = size {
                *current_bit_position += s * 8;
            }
            Ok(result)
        }
        FieldType::OptionType => {
            let result = process_option_type_functional(context, processing_ctx)?;
            // Options of primitives have the size of the primitive
            if let syn::Type::Path(tp) = context.field_type {
                if let Some(inner_type) = utils::solve_for_inner_type(tp, "Option") {
                    let field_size = utils::get_primitive_type_size(&inner_type)?;
                    *current_bit_position += field_size * 8;
                }
            }
            Ok(result)
        }
        FieldType::VarString => {
            let result = process_var_string_functional(context, processing_ctx);
            // VarString has variable size, can't track bit position
            Ok(result)
        }
        FieldType::CustomType => {
            // For custom types, we can't know the size at compile time
            // This is OK because bit fields can't come after custom types anyway
            Ok(process_custom_type_functional(context, processing_ctx))
        }
    }
}

// Functional version of handle_bits_field
fn process_bits_field_functional(
    context: &FieldContext,
    size: usize,
    processing_ctx: &crate::functional::ProcessingContext,
    bit_position: usize,
) -> Result<crate::functional::FieldProcessResult, syn::Error> {
    let field_name = &context.field_name;
    let field_type = context.field_type;

    let accessor = crate::functional::pure_helpers::create_field_accessor(field_name, false);
    let bit_sum = crate::functional::pure_helpers::create_bit_sum(size);
    let limit_check =
        crate::functional::pure_helpers::create_bit_field_limit_check(field_name, field_type, size);

    // Get the size of the underlying number type
    let number_length = utils::get_primitive_type_size(field_type)
        .map_err(|_| syn::Error::new(field_type.span(), "Type not supported"))?;

    let mask: u128 = (1 << size) - 1;

    // Check if this is a char type for special handling
    let is_char_type = if let syn::Type::Path(tp) = field_type {
        tp.path.is_ident("char")
    } else {
        false
    };

    // Use multi-byte path if:
    // 1. The underlying type is multi-byte (number_length > 1), OR
    // 2. The field is too large to ever fit in a single byte regardless of position (size > 8)
    // Note: Fields with size <= 8 can still span bytes at runtime, but we handle this differently
    let (parsing, writing) = if number_length > 1 || size > 8 {
        // Optimized multi-byte bit field handling
        let from_bytes_method = utils::get_from_bytes_method(processing_ctx.endianness);
        let to_bytes_method = utils::get_to_bytes_method(processing_ctx.endianness);

        let aligned_parsing = if is_char_type {
            // For char, we need to validate after extracting the u32 value
            crate::functional::pure_helpers::create_aligned_char_parsing(
                &from_bytes_method,
                number_length,
            )
        } else {
            crate::functional::pure_helpers::create_aligned_multibyte_parsing(
                field_type,
                &from_bytes_method,
                number_length,
            )
        };
        let unaligned_parsing = if is_char_type {
            crate::functional::pure_helpers::create_unaligned_char_parsing(
                size,
                processing_ctx.endianness,
            )
        } else {
            crate::functional::pure_helpers::create_unaligned_multibyte_parsing(
                field_type,
                size,
                processing_ctx.endianness,
            )
        };

        let parsing = quote! {
            let #field_name = {
                let byte_start = _bit_sum / 8;
                let bit_end = _bit_sum + #size;
                let bytes_needed = (bit_end + 7) / 8; // Round up to nearest byte
                if bytes_needed > bytes.len() {
                    return Err(::bebytes::BeBytesError::InsufficientData {
                        expected: bytes_needed - byte_start,
                        actual: bytes.len() - byte_start,
                    });
                }

                // Optimized: Use compile-time bit position when available
                let bit_offset = _bit_sum % 8;

                // Check if we can use compile-time optimization
                if bit_offset == 0 && (#bit_position % 8 == 0) && (#size == (#number_length * 8)) {
                    // Compile-time aligned case: direct conversion without bit manipulation
                    #aligned_parsing
                } else if bit_offset == 0 && #size == (#number_length * 8) {
                    // Runtime aligned case: direct conversion without bit manipulation
                    #aligned_parsing
                } else {
                    // Unaligned case: optimized bit extraction
                    #unaligned_parsing
                }
            };
            _bit_sum += #size;
        };

        let aligned_writing = if is_char_type {
            crate::functional::pure_helpers::create_aligned_char_writing(
                &to_bytes_method,
                number_length,
            )
        } else {
            crate::functional::pure_helpers::create_aligned_multibyte_writing(
                field_type,
                &to_bytes_method,
                number_length,
            )
        };
        let unaligned_writing = if is_char_type {
            crate::functional::pure_helpers::create_unaligned_char_writing(
                size,
                processing_ctx.endianness,
            )
        } else {
            crate::functional::pure_helpers::create_unaligned_multibyte_writing(
                field_type,
                size,
                processing_ctx.endianness,
            )
        };

        let writing = if is_char_type {
            quote! {
                if (#field_name as u32) > #mask as u32 {
                    panic!(
                        "Value {} for field {} exceeds the maximum allowed value {}.",
                        #field_name as u32,
                        stringify!(#field_name),
                        #mask
                    );
                }

                // Optimized: Use compile-time bit position when available
                let value = #field_name as u32 & #mask as u32;
            }
        } else {
            quote! {
                if #field_name > #mask as #field_type {
                    panic!(
                        "Value {} for field {} exceeds the maximum allowed value {}.",
                        #field_name,
                        stringify!(#field_name),
                        #mask
                    );
                }

                // Optimized: Use compile-time bit position when available
                let value = #field_name & #mask as #field_type;
            }
        };

        let writing = quote! {
            #writing
            let byte_start = _bit_sum / 8;
            let bit_offset = _bit_sum % 8;
            // Check if we can use compile-time optimization
            if bit_offset == 0 && (#bit_position % 8 == 0) && (#size == (#number_length * 8)) {
                // Compile-time aligned case: direct byte insertion
                #aligned_writing
            } else if bit_offset == 0 && #size == (#number_length * 8) {
                // Runtime aligned case: direct byte insertion
                #aligned_writing
            } else {
                // Unaligned case: optimized bit insertion
                #unaligned_writing
            }
            _bit_sum += #size;
        };

        (parsing, writing)
    } else {
        // Single byte bit field - generate inline code that uses _bit_sum
        let mask: u128 = (1 << size) - 1;

        let parsing = crate::functional::pure_helpers::create_single_byte_bit_parsing(
            field_name,
            field_type,
            size,
            mask,
            processing_ctx.endianness,
        );

        let writing = crate::functional::pure_helpers::create_single_byte_bit_writing(
            field_name,
            size,
            mask,
            processing_ctx.endianness,
        );

        (parsing, writing)
    };

    Ok(crate::functional::FieldProcessResult::new(
        limit_check,
        parsing,
        writing,
        accessor,
        bit_sum,
    ))
}

// Functional version of handle_primitive_type
fn process_primitive_type_functional(
    context: &FieldContext,
    processing_ctx: &crate::functional::ProcessingContext,
) -> Result<crate::functional::FieldProcessResult, syn::Error> {
    let field_name = &context.field_name;
    let field_type = context.field_type;

    let field_size = utils::get_primitive_type_size(field_type)?;

    let accessor = crate::functional::pure_helpers::create_field_accessor(field_name, false);
    let bit_sum = crate::functional::pure_helpers::create_byte_bit_sum(field_size);

    let parsing_tokens = vec![
        crate::functional::pure_helpers::create_byte_indices(field_size),
        crate::functional::pure_helpers::create_primitive_parsing(
            field_name,
            field_type,
            processing_ctx.endianness,
        )?,
    ];
    let parsing = quote! { #(#parsing_tokens)* };

    let writing = crate::functional::pure_helpers::create_primitive_writing(
        field_name,
        field_type,
        processing_ctx.endianness,
    )?;

    Ok(crate::functional::FieldProcessResult::new(
        quote! {},
        parsing,
        writing,
        accessor,
        bit_sum,
    ))
}

// Functional version of handle_array
fn process_array_functional(
    context: &FieldContext,
    length: usize,
    _processing_ctx: &crate::functional::ProcessingContext,
) -> Result<crate::functional::FieldProcessResult, syn::Error> {
    let field_name = &context.field_name;
    let field_type = context.field_type;

    if let syn::Type::Array(tp) = field_type {
        if let syn::Type::Path(elem) = &*tp.elem {
            let segments = &elem.path.segments;
            if segments.len() == 1 && segments[0].ident == "u8" {
                let accessor =
                    crate::functional::pure_helpers::create_field_accessor(field_name, true);
                let bit_sum = crate::functional::pure_helpers::create_byte_bit_sum(length);

                let parsing = quote! {
                    byte_index = _bit_sum / 8;
                    let mut #field_name = [0u8; #length];
                    #field_name.copy_from_slice(&bytes[byte_index..#length + byte_index]);
                    _bit_sum += 8 * #length;
                };

                let writing = quote! {
                    // Optimized: Reserve capacity to avoid multiple reallocations
                    bytes.reserve(#length);
                    bytes.extend_from_slice(&#field_name);
                    _bit_sum += #length * 8;
                };

                return Ok(crate::functional::FieldProcessResult::new(
                    quote! {},
                    parsing,
                    writing,
                    accessor,
                    bit_sum,
                ));
            }
        }
    }

    Err(syn::Error::new_spanned(
        field_type,
        "Unsupported array type",
    ))
}

// Helper to generate vector parsing and writing tokens for primitive types
fn generate_primitive_vector_tokens(
    field_name: &syn::Ident,
    size: Option<usize>,
    vec_size_ident: Option<Vec<syn::Ident>>,
    is_last_field: bool,
    field: &syn::Field,
) -> Result<
    (
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
    ),
    syn::Error,
> {
    match (size, vec_size_ident) {
        (_, Some(ident_path)) => {
            let field_access_parse =
                crate::functional::pure_helpers::generate_field_access_path(&ident_path);
            Ok((
                quote! { bit_sum = 4096 * 8; },
                quote! {
                    let vec_size = #field_access_parse as usize;
                    byte_index = _bit_sum / 8;
                    let end_index = byte_index + vec_size;
                    if end_index > bytes.len() {
                        panic!("Not enough bytes to parse a vector of size {} (field: {}, byte_index: {}, bytes.len(): {})", vec_size, stringify!(#field_name), byte_index, bytes.len());
                    }
                    let #field_name = Vec::from(&bytes[byte_index..end_index]);
                    _bit_sum += vec_size * 8;
                },
                quote! {
                    bytes.reserve(#field_name.len());
                    bytes.extend_from_slice(&#field_name);
                    _bit_sum += #field_name.len() * 8;
                },
            ))
        }
        (Some(s), None) => Ok((
            crate::functional::pure_helpers::create_byte_bit_sum(s),
            quote! {
                let vec_size = #s as usize;
                byte_index = _bit_sum / 8;
                let end_index = byte_index + vec_size;
                let #field_name = Vec::from(&bytes[byte_index..end_index]);
                _bit_sum += #s * 8;
            },
            quote! {
                bytes.reserve(#field_name.len());
                bytes.extend_from_slice(&#field_name);
                _bit_sum += #field_name.len() * 8;
            },
        )),
        (None, None) => {
            if !is_last_field {
                return Err(syn::Error::new(
                    field.ty.span(),
                    "Unbounded vectors can only be used as padding at the end of a struct",
                ));
            }
            Ok((
                quote! { bit_sum = 4096 * 8; },
                quote! {
                    byte_index = _bit_sum / 8;
                    let #field_name = Vec::from(&bytes[byte_index..]);
                    _bit_sum += #field_name.len() * 8;
                },
                quote! {
                    bytes.reserve(#field_name.len());
                    bytes.extend_from_slice(&#field_name);
                    _bit_sum += #field_name.len() * 8;
                },
            ))
        }
    }
}

// Helper to generate parsing code for custom type vectors
fn generate_custom_vector_parsing(
    field_name: &syn::Ident,
    inner_type_name: &proc_macro2::TokenStream,
    try_from_bytes_method: &proc_macro2::TokenStream,
    is_last_field: bool,
    size: Option<usize>,
    vec_size_ident: Option<Vec<syn::Ident>>,
    field: &syn::Field,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    if is_last_field {
        Ok(quote! {
            let mut bytes_consumed = 0;
            while _bit_sum / 8 + bytes_consumed < bytes.len() {
                match #inner_type_name::#try_from_bytes_method(&bytes[_bit_sum / 8 + bytes_consumed..]) {
                    Ok((item, consumed)) => {
                        #field_name.push(item);
                        bytes_consumed += consumed;
                    }
                    Err(e) => break,
                }
            }
            _bit_sum += bytes_consumed * 8;
        })
    } else if let Some(vec_size) = size {
        Ok(quote! {
            let mut bytes_consumed = 0;
            for _ in 0..#vec_size {
                if _bit_sum / 8 + bytes_consumed >= bytes.len() {
                    break;
                }
                match #inner_type_name::#try_from_bytes_method(&bytes[_bit_sum / 8 + bytes_consumed..]) {
                    Ok((item, consumed)) => {
                        #field_name.push(item);
                        bytes_consumed += consumed;
                    }
                    Err(e) => return Err(e),
                }
            }
            _bit_sum += bytes_consumed * 8;
        })
    } else if let Some(ident_path) = vec_size_ident {
        let field_access_parse =
            crate::functional::pure_helpers::generate_field_access_path(&ident_path);
        Ok(quote! {
            let vec_size = #field_access_parse as usize;
            let mut bytes_consumed = 0;
            for _ in 0..vec_size {
                if _bit_sum / 8 + bytes_consumed >= bytes.len() {
                    break;
                }
                match #inner_type_name::#try_from_bytes_method(&bytes[_bit_sum / 8 + bytes_consumed..]) {
                    Ok((item, consumed)) => {
                        #field_name.push(item);
                        bytes_consumed += consumed;
                    }
                    Err(e) => return Err(e),
                }
            }
            _bit_sum += bytes_consumed * 8;
        })
    } else {
        Err(syn::Error::new(field.ty.span(), "Vectors of custom types need size information. Use #[With(size(n))] or #[FromField(field_name)]"))
    }
}

// Functional version of handle_vector
fn process_vector_functional(
    context: &FieldContext,
    size: Option<usize>,
    vec_size_ident: Option<Vec<syn::Ident>>,
    processing_ctx: &crate::functional::ProcessingContext,
) -> Result<crate::functional::FieldProcessResult, syn::Error> {
    let field_name = &context.field_name;
    let field_type = context.field_type;
    let field = context.field;
    let is_last_field = context.is_last_field;

    let accessor = crate::functional::pure_helpers::create_field_accessor(field_name, true);

    if let syn::Type::Path(tp) = field_type {
        if let Some(syn::Type::Path(ref inner_tp)) = utils::solve_for_inner_type(tp, "Vec") {
            if utils::is_primitive_type(inner_tp) {
                let (bit_sum, parsing, writing) = generate_primitive_vector_tokens(
                    field_name,
                    size,
                    vec_size_ident,
                    is_last_field,
                    field,
                )?;

                return Ok(crate::functional::FieldProcessResult::new(
                    quote! {},
                    parsing,
                    writing,
                    accessor,
                    bit_sum,
                ));
            }

            // Handle vector of custom types
            let inner_type_path = &inner_tp.path;
            let inner_type_name = quote! { #inner_type_path };

            let try_from_bytes_method = utils::get_try_from_bytes_method(processing_ctx.endianness);
            let to_bytes_method = utils::get_to_bytes_method(processing_ctx.endianness);

            let parsing_init = quote! {
                let mut #field_name = Vec::new();
            };

            let parsing_loop = generate_custom_vector_parsing(
                field_name,
                &inner_type_name,
                &try_from_bytes_method,
                is_last_field,
                size,
                vec_size_ident,
                field,
            )?;

            let parsing = quote! {
                #parsing_init
                #parsing_loop
            };

            let writing = quote! {
                // Optimized: Pre-calculate total size to avoid multiple reallocations
                let total_size = #field_name.iter().map(|item| {
                    BeBytes::#to_bytes_method(item).len()
                }).sum::<usize>();
                bytes.reserve(total_size);

                for item in &#field_name {
                    let item_bytes = BeBytes::#to_bytes_method(item);
                    bytes.extend_from_slice(&item_bytes);
                    _bit_sum += item_bytes.len() * 8;
                }
            };

            return Ok(crate::functional::FieldProcessResult::new(
                quote! {},
                parsing,
                writing,
                accessor,
                quote! { bit_sum = 4096 * 8; }, // Variable size
            ));
        }
    }

    Err(syn::Error::new_spanned(field_type, "Not a vector type"))
}

// Functional version of handle_option_type
fn process_option_type_functional(
    context: &FieldContext,
    processing_ctx: &crate::functional::ProcessingContext,
) -> Result<crate::functional::FieldProcessResult, syn::Error> {
    let field_name = &context.field_name;
    let field_type = context.field_type;

    if let syn::Type::Path(tp) = field_type {
        if let Some(inner_type) = utils::solve_for_inner_type(tp, "Option") {
            if let syn::Type::Path(inner_tp) = &inner_type {
                if utils::is_primitive_type(inner_tp) {
                    let field_size = utils::get_primitive_type_size(&inner_type)?;

                    let accessor =
                        crate::functional::pure_helpers::create_field_accessor(field_name, false);
                    let bit_sum = crate::functional::pure_helpers::create_byte_bit_sum(field_size);

                    let from_bytes_method = utils::get_from_bytes_method(processing_ctx.endianness);
                    let to_bytes_method = utils::get_to_bytes_method(processing_ctx.endianness);

                    let parsing = quote! {
                        byte_index = _bit_sum / 8;
                        end_byte_index = byte_index + #field_size;
                        _bit_sum += 8 * #field_size;
                        let #field_name = if bytes[byte_index..end_byte_index] == [0_u8; #field_size] {
                            None
                        } else {
                            Some(<#inner_tp>::#from_bytes_method({
                                let slice = &bytes[byte_index..end_byte_index];
                                let mut arr = [0; #field_size];
                                arr.copy_from_slice(slice);
                                arr
                            }))
                        };
                    };

                    let writing = quote! {
                        let bytes_data = &#field_name.unwrap_or(0).#to_bytes_method();
                        // Optimized: Reserve capacity to avoid reallocation
                        bytes.reserve(bytes_data.len());
                        bytes.extend_from_slice(bytes_data);
                        _bit_sum += bytes_data.len() * 8;
                    };

                    return Ok(crate::functional::FieldProcessResult::new(
                        quote! {},
                        parsing,
                        writing,
                        accessor,
                        bit_sum,
                    ));
                }
            }
        }
    }

    Err(syn::Error::new_spanned(
        field_type,
        "Unsupported Option type",
    ))
}

// Functional version of handle_custom_type
fn process_custom_type_functional(
    context: &FieldContext,
    processing_ctx: &crate::functional::ProcessingContext,
) -> crate::functional::FieldProcessResult {
    let field_name = &context.field_name;
    let field_type = context.field_type;

    let needs_owned = !utils::is_copy(field_type);
    let accessor = crate::functional::pure_helpers::create_field_accessor(field_name, needs_owned);

    let bit_sum = quote! {
        bit_sum += 8 * #field_type::field_size();
    };

    let try_from_bytes_method = utils::get_try_from_bytes_method(processing_ctx.endianness);
    let to_bytes_method = utils::get_to_bytes_method(processing_ctx.endianness);

    let parsing = quote_spanned! { context.field.span() =>
        byte_index = _bit_sum / 8;
        let predicted_size = #field_type::field_size();
        end_byte_index = usize::min(bytes.len(), byte_index + predicted_size);
        let (#field_name, bytes_read) = #field_type::#try_from_bytes_method(&bytes[byte_index..end_byte_index])?;
        _bit_sum += bytes_read * 8;
    };

    let writing = quote_spanned! { context.field.span() =>
        let bytes_data = &BeBytes::#to_bytes_method(&#field_name);
        // Optimized: Reserve capacity to avoid reallocation
        bytes.reserve(bytes_data.len());
        bytes.extend_from_slice(bytes_data);
        _bit_sum += bytes_data.len() * 8;
    };

    crate::functional::FieldProcessResult::new(quote! {}, parsing, writing, accessor, bit_sum)
}

// Functional version for VarString fields
fn process_var_string_functional(
    context: &FieldContext,
    processing_ctx: &crate::functional::ProcessingContext,
) -> crate::functional::FieldProcessResult {
    let field_name = &context.field_name;
    let field_type = context.field_type;

    let needs_owned = !utils::is_copy(field_type);
    let accessor = crate::functional::pure_helpers::create_field_accessor(field_name, needs_owned);

    // VarString is variable size, so we can't predict bit_sum at compile time
    // We don't add anything to bit_sum because the size is variable
    let bit_sum = quote! {
        // VarString fields don't contribute to compile-time size calculation
    };

    let try_from_bytes_method = utils::get_try_from_bytes_method(processing_ctx.endianness);
    let to_bytes_method = utils::get_to_bytes_method(processing_ctx.endianness);

    let parsing = quote_spanned! { context.field.span() =>
        byte_index = _bit_sum / 8;
        let (temp_value, bytes_consumed) = #field_type::#try_from_bytes_method(&bytes[byte_index..])?;
        let #field_name = temp_value;
        _bit_sum += bytes_consumed * 8;
    };

    let writing = quote_spanned! { context.field.span() =>
        let bytes_data = &BeBytes::#to_bytes_method(&#field_name);
        // Optimized: Reserve capacity to avoid reallocation
        bytes.reserve(bytes_data.len());
        bytes.extend_from_slice(bytes_data);
        _bit_sum += bytes_data.len() * 8;
    };

    crate::functional::FieldProcessResult::new(quote! {}, parsing, writing, accessor, bit_sum)
}
