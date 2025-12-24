use crate::functional::pure_helpers::{create_field_accessor, AccessorMode};
use crate::{attrs, utils};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

/// Convert Vec-based writing code to BufMut-based direct writing
fn convert_to_direct_writing(writing_code: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    // For most field types, use fallback approach with temp Vec
    // This avoids variable name conflicts while maintaining correctness
    quote! {
        {
            let mut field_bytes = Vec::new();
            {
                let bytes = &mut field_bytes; // Create alias to avoid conflicts with 'bytes' crate
                #writing_code
            }
            buf.put_slice(&field_bytes);
        }
    }
}

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

enum FieldType {
    BitsField(usize), // only size, position is auto-calculated
    PrimitiveType,
    Array(usize),                                     // array_length
    Vector(Option<usize>, Option<Vec<syn::Ident>>),   // size, vec_size_ident
    String(Option<usize>, Option<Vec<syn::Ident>>),   // size, string_size_ident
    SizeExpression(crate::size_expr::SizeExpression), // expression-based sizing
    OptionType,
    CustomType,
    UntilMarker(u8), // Read Vec<T> until marker byte
    AfterMarker(u8), // Read remaining bytes after marker
    VecOfVecsWithMarker(Option<usize>, Option<Vec<syn::Ident>>, u8), // size, field_path, marker
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
    pub direct_writing: Vec<proc_macro2::TokenStream>, // New: direct buffer writing
    pub named_fields: Vec<proc_macro2::TokenStream>,
    pub total_size: usize,
}

pub struct StructContext<'a> {
    pub field_limit_check: &'a mut Vec<proc_macro2::TokenStream>,
    pub errors: &'a mut Vec<proc_macro2::TokenStream>,
    pub field_parsing: &'a mut Vec<proc_macro2::TokenStream>,
    pub bit_sum: &'a mut Vec<proc_macro2::TokenStream>,
    pub field_writing: &'a mut Vec<proc_macro2::TokenStream>,
    pub direct_writing: &'a mut Vec<proc_macro2::TokenStream>, // New: direct buffer writing
    pub named_fields: &'a mut Vec<proc_macro2::TokenStream>,
    pub fields: &'a syn::FieldsNamed,
    pub endianness: crate::consts::Endianness,
    pub has_bit_fields: &'a mut bool, // Track if any fields have bit attributes
}

fn handle_marker_attribute(
    context: &FieldContext,
    _marker: u8,
    marker_type: &str,
    errors: &mut Vec<proc_macro2::TokenStream>,
    field_type: FieldType,
) -> Option<FieldType> {
    if let syn::Type::Path(tp) = context.field_type {
        if !tp.path.segments.is_empty() && tp.path.segments[0].ident == "Vec" {
            return Some(field_type);
        }
    }
    errors.push(
        syn::Error::new(
            context.field_type.span(),
            format!("{marker_type} can only be used with Vec types"),
        )
        .to_compile_error(),
    );
    None
}

// Helper function to handle bits field processing
fn handle_bits_field(
    context: &FieldContext,
    size: Option<usize>,
    errors: &mut Vec<proc_macro2::TokenStream>,
    has_bit_fields: &mut bool,
) -> Option<FieldType> {
    *has_bit_fields = true;
    if let Some(size) = size {
        // Validate bit count doesn't exceed capacity (also validates type is bitfield-compatible)
        if let Ok(max_bits) = utils::get_primitive_type_max_bits(context.field_type) {
            if size > max_bits {
                errors.push(syn::Error::new(
                    context.field.span(),
                    format!("bits attribute specifies {size} bits, but type can only hold {max_bits} bits"),
                ).to_compile_error());
                return None;
            }
        } else {
            errors.push(syn::Error::new(
                context.field_type.span(),
                "Unsupported type for bits attribute. Only integer types (u8, i8, u16, i16, u32, i32, u64, i64, u128, i128) and char are supported. Note: f32, f64, and bool cannot be used with bit fields",
            ).to_compile_error());
            return None;
        }
        return Some(FieldType::BitsField(size));
    }
    // Empty #[bits()] is no longer supported
    errors.push(syn::Error::new(
        context.field.span(),
        "Size required for bits attribute. Use #[bits(N)] where N is the number of bits needed for your field",
    ).to_compile_error());
    None
}

// Helper function to handle Vec<Vec<u8>> with markers
fn handle_vec_of_vecs_marker(
    context: &FieldContext,
    marker: u8,
    is_until: bool,
    size: Option<usize>,
    vec_size_ident: Option<Vec<syn::Ident>>,
    errors: &mut Vec<proc_macro2::TokenStream>,
) -> Option<FieldType> {
    if let syn::Type::Path(tp) = context.field_type {
        if utils::is_vec_of_vec_u8(tp) {
            if is_until {
                // Vec<Vec<u8>> with UntilMarker requires size control
                if size.is_some() || vec_size_ident.is_some() {
                    return Some(FieldType::VecOfVecsWithMarker(size, vec_size_ident, marker));
                }
                errors.push(syn::Error::new(
                    context.field_type.span(),
                    "Vec<Vec<u8>> with UntilMarker requires size control via #[With(size(N))] or #[FromField(field_name)]"
                ).to_compile_error());
                return None;
            }
            // AfterMarker is not supported for Vec<Vec<u8>>
            errors.push(
                syn::Error::new(
                    context.field_type.span(),
                    "AfterMarker is not supported with Vec<Vec<u8>>. Use UntilMarker instead",
                )
                .to_compile_error(),
            );
            return None;
        }
    }
    None
}

fn determine_field_type(
    context: &FieldContext,
    attrs: &[syn::Attribute],
    errors: &mut Vec<proc_macro2::TokenStream>,
    has_bit_fields: &mut bool,
) -> Option<FieldType> {
    let mut bits_attribute_present = false;
    let (size, vec_size_ident, size_expression, until_marker, after_marker) =
        attrs::parse_attributes_with_expressions(attrs, &mut bits_attribute_present, errors);

    // Check for marker attributes
    if let Some(marker) = until_marker {
        // Check if this is Vec<Vec<u8>> with size control
        if let Some(field_type) = handle_vec_of_vecs_marker(
            context,
            marker,
            true, // is_until
            size,
            vec_size_ident.clone(),
            errors,
        ) {
            return Some(field_type);
        }

        return handle_marker_attribute(
            context,
            marker,
            "UntilMarker",
            errors,
            FieldType::UntilMarker(marker),
        );
    }

    if let Some(marker) = after_marker {
        // Check if this is Vec<Vec<u8>> - AfterMarker is not supported
        if let Some(field_type) = handle_vec_of_vecs_marker(
            context,
            marker,
            false, // is_until = false (after_marker)
            size,
            vec_size_ident.clone(),
            errors,
        ) {
            return Some(field_type);
        }

        return handle_marker_attribute(
            context,
            marker,
            "AfterMarker",
            errors,
            FieldType::AfterMarker(marker),
        );
    }

    if bits_attribute_present {
        return handle_bits_field(context, size, errors, has_bit_fields);
    }

    // Check for size expression
    if let Some(expr) = size_expression {
        // Size expressions are only supported for Vec<u8> and String types for now
        match context.field_type {
            syn::Type::Path(tp) if !tp.path.segments.is_empty() => {
                let segment = &tp.path.segments[0];
                match &segment.ident {
                    ident if ident == "Vec" || ident == "String" => {
                        return Some(FieldType::SizeExpression(expr));
                    }
                    _ => {
                        let error = syn::Error::new(
                            context.field_type.span(),
                            "Size expressions are only supported for Vec<u8> and String types",
                        );
                        errors.push(error.to_compile_error());
                        return None;
                    }
                }
            }
            _ => {
                let error = syn::Error::new(
                    context.field_type.span(),
                    "Size expressions are only supported for Vec<u8> and String types",
                );
                errors.push(error.to_compile_error());
                return None;
            }
        }
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
                ident if ident == "String" => Some(FieldType::String(size, vec_size_ident)),
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

        if let Some(field_type) = determine_field_type(
            &field_context,
            &field.attrs,
            &mut errors,
            context.has_bit_fields,
        ) {
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
    context.direct_writing.extend(field_data.direct_writing);
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
            if let syn::Type::Path(tp) = context.field_type {
                if let Some(inner_type) = utils::solve_for_inner_type(tp, "Option") {
                    let inner_size = match &inner_type {
                        syn::Type::Path(_) => utils::get_primitive_type_size(&inner_type)?,
                        syn::Type::Array(arr) => {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Int(lit_int),
                                ..
                            }) = &arr.len
                            {
                                lit_int.base10_parse()?
                            } else {
                                return Err(syn::Error::new_spanned(
                                    &inner_type,
                                    "Array length must be a literal",
                                ));
                            }
                        }
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &inner_type,
                                "Unsupported Option inner type",
                            ));
                        }
                    };
                    *current_bit_position += (inner_size + 1) * 8;
                }
            }
            Ok(result)
        }
        FieldType::String(size, string_size_ident) => {
            let result =
                process_string_functional(context, size, string_size_ident, processing_ctx)?;
            // Strings have variable size, but we need to track something for bit field positioning
            if let Some(s) = size {
                *current_bit_position += s * 8;
            }
            Ok(result)
        }
        FieldType::SizeExpression(size_expr) => {
            let result = process_size_expression_functional(context, &size_expr, processing_ctx)?;
            // Size expressions have variable size, so we can't update bit position
            // This means no bit fields can come after size expression fields
            Ok(result)
        }
        FieldType::CustomType => {
            // For custom types, we can't know the size at compile time
            // This is OK because bit fields can't come after custom types anyway
            Ok(process_custom_type_functional(context, processing_ctx))
        }
        FieldType::UntilMarker(marker) => {
            // UntilMarker fields have dynamic size
            Ok(process_until_marker_functional(
                context,
                marker,
                processing_ctx,
            ))
        }
        FieldType::AfterMarker(marker) => {
            // AfterMarker fields consume remaining bytes
            Ok(process_after_marker_functional(
                context,
                marker,
                processing_ctx,
            ))
        }
        FieldType::VecOfVecsWithMarker(size, field_path, marker) => {
            // Vec<Vec<u8>> with marker delimiting and size control
            Ok(process_vec_of_vecs_with_marker_functional(
                context,
                size,
                field_path,
                marker,
                processing_ctx,
            ))
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

    let accessor = create_field_accessor(field_name, AccessorMode::Copy);
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

    let (parsing, writing) = if number_length > 1 || size > 8 {
        let ctx = MultiByteBitFieldCtx {
            field_name,
            field_type,
            size,
            number_length,
            bit_position,
            is_char_type,
            mask,
            processing_ctx,
        };
        generate_multibyte_bit_field(&ctx)
    } else {
        generate_single_byte_bit_field(
            field_name,
            field_type,
            size,
            mask,
            bit_position,
            processing_ctx,
        )
    };

    let direct_writing = convert_to_direct_writing(&writing);
    Ok(crate::functional::FieldProcessResult::new(
        limit_check,
        parsing,
        writing,
        direct_writing,
        accessor,
        bit_sum,
    ))
}

// Context for multi-byte bit field generation
struct MultiByteBitFieldCtx<'a> {
    field_name: &'a syn::Ident,
    field_type: &'a syn::Type,
    size: usize,
    number_length: usize,
    bit_position: usize,
    is_char_type: bool,
    mask: u128,
    processing_ctx: &'a crate::functional::ProcessingContext,
}

// Generate code for multi-byte bit fields
fn generate_multibyte_bit_field(
    ctx: &MultiByteBitFieldCtx,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let from_bytes_method = utils::get_from_bytes_method(ctx.processing_ctx.endianness);
    let to_bytes_method = utils::get_to_bytes_method(ctx.processing_ctx.endianness);

    let (aligned_parsing, unaligned_parsing) = generate_parsing_tokens(
        ctx.field_type,
        ctx.size,
        ctx.number_length,
        ctx.is_char_type,
        &from_bytes_method,
        ctx.processing_ctx,
    );

    let field_name = ctx.field_name;
    let size = ctx.size;
    let bit_position = ctx.bit_position;
    let number_length = ctx.number_length;

    let parsing = quote! {
        let #field_name = {
            let byte_start = _bit_sum / 8;
            let bit_end = _bit_sum + #size;
            let bytes_needed = bit_end.div_ceil(8);
            if bytes_needed > bytes.len() {
                return Err(::bebytes::BeBytesError::InsufficientData {
                    expected: bytes_needed - byte_start,
                    actual: bytes.len() - byte_start,
                });
            }

            let bit_offset = _bit_sum % 8;
            if bit_offset == 0 && (#bit_position % 8 == 0) && (#size == (#number_length * 8)) {
                #aligned_parsing
            } else if bit_offset == 0 && #size == (#number_length * 8) {
                #aligned_parsing
            } else {
                #unaligned_parsing
            }
        };
        _bit_sum += #size;
    };

    let (aligned_writing, unaligned_writing) = generate_writing_tokens(
        ctx.field_type,
        ctx.size,
        ctx.number_length,
        ctx.is_char_type,
        &to_bytes_method,
        ctx.processing_ctx,
    );

    let value_prep =
        generate_value_preparation(ctx.field_name, ctx.field_type, ctx.is_char_type, ctx.mask);

    let writing = quote! {
        #value_prep
        let byte_start = _bit_sum / 8;
        let bit_offset = _bit_sum % 8;
        if bit_offset == 0 && (#bit_position % 8 == 0) && (#size == (#number_length * 8)) {
            #aligned_writing
        } else if bit_offset == 0 && #size == (#number_length * 8) {
            #aligned_writing
        } else {
            #unaligned_writing
        }
        _bit_sum += #size;
    };

    (parsing, writing)
}

fn generate_single_byte_bit_field(
    field_name: &syn::Ident,
    field_type: &syn::Type,
    size: usize,
    mask: u128,
    bit_position: usize,
    processing_ctx: &crate::functional::ProcessingContext,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let bit_offset = bit_position % 8;
    let spans_bytes = bit_offset + size > 8;

    let parsing = crate::functional::pure_helpers::create_single_byte_bit_parsing(
        field_name,
        field_type,
        size,
        mask,
        bit_offset,
        spans_bytes,
        processing_ctx.endianness,
    );

    let writing = crate::functional::pure_helpers::create_single_byte_bit_writing(
        field_name,
        size,
        mask,
        bit_offset,
        spans_bytes,
        processing_ctx.endianness,
    );

    (parsing, writing)
}

// Helper to generate parsing tokens
fn generate_parsing_tokens(
    field_type: &syn::Type,
    size: usize,
    number_length: usize,
    is_char_type: bool,
    from_bytes_method: &proc_macro2::TokenStream,
    processing_ctx: &crate::functional::ProcessingContext,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let aligned_parsing = if is_char_type {
        crate::functional::pure_helpers::create_aligned_char_parsing(
            from_bytes_method,
            number_length,
        )
    } else {
        crate::functional::pure_helpers::create_aligned_multibyte_parsing(
            field_type,
            from_bytes_method,
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

    (aligned_parsing, unaligned_parsing)
}

// Helper to generate writing tokens
fn generate_writing_tokens(
    field_type: &syn::Type,
    size: usize,
    number_length: usize,
    is_char_type: bool,
    to_bytes_method: &proc_macro2::TokenStream,
    processing_ctx: &crate::functional::ProcessingContext,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let aligned_writing = if is_char_type {
        crate::functional::pure_helpers::create_aligned_char_writing(to_bytes_method, number_length)
    } else {
        crate::functional::pure_helpers::create_aligned_multibyte_writing(
            field_type,
            to_bytes_method,
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

    (aligned_writing, unaligned_writing)
}

// Helper to generate value preparation code
fn generate_value_preparation(
    field_name: &syn::Ident,
    field_type: &syn::Type,
    is_char_type: bool,
    mask: u128,
) -> proc_macro2::TokenStream {
    if is_char_type {
        quote! {
            if (#field_name as u32) > #mask as u32 {
                panic!(
                    "Value {} for field {} exceeds the maximum allowed value {}.",
                    #field_name as u32,
                    stringify!(#field_name),
                    #mask
                );
            }
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
            let value = #field_name & #mask as #field_type;
        }
    }
}

fn process_primitive_type_functional(
    context: &FieldContext,
    processing_ctx: &crate::functional::ProcessingContext,
) -> Result<crate::functional::FieldProcessResult, syn::Error> {
    let field_name = &context.field_name;
    let field_type = context.field_type;

    let field_size = utils::get_primitive_type_size(field_type)?;

    let accessor = create_field_accessor(field_name, AccessorMode::Copy);
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

    let direct_writing = crate::functional::pure_helpers::create_primitive_direct_writing(
        field_name,
        field_type,
        processing_ctx.endianness,
    )?;

    Ok(crate::functional::FieldProcessResult::new(
        quote! {},
        parsing,
        writing,
        direct_writing,
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
                let accessor = create_field_accessor(field_name, AccessorMode::Copy);
                let bit_sum = crate::functional::pure_helpers::create_byte_bit_sum(length);

                let parsing = quote! {
                    byte_index = _bit_sum / 8;
                    let mut #field_name = [0u8; #length];
                    #field_name.copy_from_slice(&bytes[byte_index..#length + byte_index]);
                    _bit_sum += 8 * #length;
                };

                let writing = quote! {
                    bytes.reserve(#length);
                    bytes.extend_from_slice(&#field_name);
                    _bit_sum += #length * 8;
                };

                let direct_writing = quote! {
                    buf.put_slice(&#field_name);
                };

                return Ok(crate::functional::FieldProcessResult::new(
                    quote! {},
                    parsing,
                    writing,
                    direct_writing,
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

    let accessor = create_field_accessor(field_name, AccessorMode::Clone);

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

                let direct_writing = quote! {
                    buf.put_slice(&#field_name);
                };

                return Ok(crate::functional::FieldProcessResult::new(
                    quote! {},
                    parsing,
                    writing,
                    direct_writing,
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

            let encode_method = utils::get_encode_to_method(processing_ctx.endianness);
            let direct_writing = quote! {
                for item in &#field_name {
                    let _ = BeBytes::#encode_method(item, buf);
                }
            };
            return Ok(crate::functional::FieldProcessResult::new(
                quote! {},
                parsing,
                writing,
                direct_writing,
                accessor,
                quote! { bit_sum = 4096 * 8; }, // Variable size
            ));
        }
    }

    Err(syn::Error::new_spanned(field_type, "Not a vector type"))
}

fn option_parse_special(
    inner_tp: &syn::TypePath,
    endianness: crate::consts::Endianness,
) -> Option<proc_macro2::TokenStream> {
    if inner_tp.path.is_ident("bool") {
        return Some(quote! { Some(bytes[value_start] != 0) });
    }
    if inner_tp.path.is_ident("char") {
        let from_bytes = match endianness {
            crate::consts::Endianness::Big => quote! { u32::from_be_bytes },
            crate::consts::Endianness::Little => quote! { u32::from_le_bytes },
        };
        return Some(quote! {{
            let char_value = #from_bytes([
                bytes[value_start], bytes[value_start + 1],
                bytes[value_start + 2], bytes[value_start + 3]
            ]);
            Some(char::from_u32(char_value)
                .ok_or_else(|| ::bebytes::BeBytesError::InvalidChar {
                    value: char_value,
                })?)
        }});
    }
    if inner_tp.path.is_ident("f32") {
        let from_bytes = match endianness {
            crate::consts::Endianness::Big => quote! { f32::from_be_bytes },
            crate::consts::Endianness::Little => quote! { f32::from_le_bytes },
        };
        return Some(quote! {
            Some(#from_bytes([
                bytes[value_start], bytes[value_start + 1],
                bytes[value_start + 2], bytes[value_start + 3]
            ]))
        });
    }
    if inner_tp.path.is_ident("f64") {
        let from_bytes = match endianness {
            crate::consts::Endianness::Big => quote! { f64::from_be_bytes },
            crate::consts::Endianness::Little => quote! { f64::from_le_bytes },
        };
        return Some(quote! {
            Some(#from_bytes([
                bytes[value_start], bytes[value_start + 1],
                bytes[value_start + 2], bytes[value_start + 3],
                bytes[value_start + 4], bytes[value_start + 5],
                bytes[value_start + 6], bytes[value_start + 7]
            ]))
        });
    }
    None
}

fn option_parse_integer(
    inner_tp: &syn::TypePath,
    field_size: usize,
    from_bytes_method: &proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    match field_size {
        1 => Ok(quote! { Some(bytes[value_start] as #inner_tp) }),
        2 => Ok(quote! {
            Some(<#inner_tp>::#from_bytes_method([bytes[value_start], bytes[value_start + 1]]))
        }),
        4 => Ok(quote! {
            Some(<#inner_tp>::#from_bytes_method([
                bytes[value_start], bytes[value_start + 1],
                bytes[value_start + 2], bytes[value_start + 3]
            ]))
        }),
        8 => Ok(quote! {
            Some(<#inner_tp>::#from_bytes_method([
                bytes[value_start], bytes[value_start + 1], bytes[value_start + 2], bytes[value_start + 3],
                bytes[value_start + 4], bytes[value_start + 5], bytes[value_start + 6], bytes[value_start + 7]
            ]))
        }),
        16 => Ok(quote! {
            Some(<#inner_tp>::#from_bytes_method([
                bytes[value_start], bytes[value_start + 1], bytes[value_start + 2], bytes[value_start + 3],
                bytes[value_start + 4], bytes[value_start + 5], bytes[value_start + 6], bytes[value_start + 7],
                bytes[value_start + 8], bytes[value_start + 9], bytes[value_start + 10], bytes[value_start + 11],
                bytes[value_start + 12], bytes[value_start + 13], bytes[value_start + 14], bytes[value_start + 15]
            ]))
        }),
        _ => Err(syn::Error::new_spanned(
            inner_tp,
            "Unsupported primitive type size for Option",
        )),
    }
}

fn create_option_inner_parsing(
    inner_tp: &syn::TypePath,
    field_size: usize,
    endianness: crate::consts::Endianness,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    if let Some(ts) = option_parse_special(inner_tp, endianness) {
        return Ok(ts);
    }
    let from_bytes_method = utils::get_from_bytes_method(endianness);
    option_parse_integer(inner_tp, field_size, &from_bytes_method)
}

fn create_option_inner_writing(
    inner_tp: &syn::TypePath,
    endianness: crate::consts::Endianness,
) -> proc_macro2::TokenStream {
    if inner_tp.path.is_ident("bool") {
        return quote! {
            ::bebytes::BufMut::put_u8(bytes, if inner_val { 1 } else { 0 });
        };
    }

    if inner_tp.path.is_ident("char") {
        return match endianness {
            crate::consts::Endianness::Big => quote! {
                bytes.extend_from_slice(&(inner_val as u32).to_be_bytes());
            },
            crate::consts::Endianness::Little => quote! {
                bytes.extend_from_slice(&(inner_val as u32).to_le_bytes());
            },
        };
    }

    if inner_tp.path.is_ident("f32") || inner_tp.path.is_ident("f64") {
        return match endianness {
            crate::consts::Endianness::Big => quote! {
                bytes.extend_from_slice(&inner_val.to_be_bytes());
            },
            crate::consts::Endianness::Little => quote! {
                bytes.extend_from_slice(&inner_val.to_le_bytes());
            },
        };
    }

    let to_bytes_method = utils::get_to_bytes_method(endianness);
    quote! {
        bytes.extend_from_slice(&inner_val.#to_bytes_method());
    }
}

fn process_option_array(
    field_name: &syn::Ident,
    arr: &syn::TypeArray,
) -> Result<crate::functional::FieldProcessResult, syn::Error> {
    if let syn::Type::Path(elem) = &*arr.elem {
        if elem.path.is_ident("u8") {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit_int),
                ..
            }) = &arr.len
            {
                let array_len: usize = lit_int.base10_parse()?;
                let total_size = array_len + 1;

                let accessor = create_field_accessor(field_name, AccessorMode::Copy);
                let bit_sum = crate::functional::pure_helpers::create_byte_bit_sum(total_size);

                let parsing = quote! {
                    byte_index = _bit_sum / 8;
                    end_byte_index = byte_index + #total_size;
                    if end_byte_index > bytes.len() {
                        return Err(::bebytes::BeBytesError::InsufficientData {
                            expected: end_byte_index,
                            actual: bytes.len(),
                        });
                    }
                    let #field_name = match bytes[byte_index] {
                        0x00 => None,
                        0x01 => {
                            let value_start = byte_index + 1;
                            let mut arr = [0u8; #array_len];
                            arr.copy_from_slice(&bytes[value_start..value_start + #array_len]);
                            Some(arr)
                        }
                        tag => return Err(::bebytes::BeBytesError::InvalidDiscriminant {
                            value: tag,
                            type_name: "Option",
                        }),
                    };
                    _bit_sum += 8 * #total_size;
                };

                let writing = quote! {
                    match #field_name {
                        None => {
                            bytes.reserve(#total_size);
                            ::bebytes::BufMut::put_u8(bytes, 0x00);
                            bytes.extend_from_slice(&[0u8; #array_len]);
                        }
                        Some(arr) => {
                            bytes.reserve(#total_size);
                            ::bebytes::BufMut::put_u8(bytes, 0x01);
                            bytes.extend_from_slice(&arr);
                        }
                    }
                    _bit_sum += #total_size * 8;
                };

                let direct_writing = quote! {
                    match #field_name {
                        None => {
                            buf.put_u8(0x00);
                            buf.put_slice(&[0u8; #array_len]);
                        }
                        Some(arr) => {
                            buf.put_u8(0x01);
                            buf.put_slice(&arr);
                        }
                    }
                };

                return Ok(crate::functional::FieldProcessResult::new(
                    quote! {},
                    parsing,
                    writing,
                    direct_writing,
                    accessor,
                    bit_sum,
                ));
            }
        }
    }
    Err(syn::Error::new_spanned(
        arr,
        "Only [u8; N] arrays are supported in Option",
    ))
}

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
                    let total_size = field_size + 1;

                    let accessor = create_field_accessor(field_name, AccessorMode::Copy);
                    let bit_sum = crate::functional::pure_helpers::create_byte_bit_sum(total_size);

                    let value_parsing = create_option_inner_parsing(
                        inner_tp,
                        field_size,
                        processing_ctx.endianness,
                    )?;

                    let parsing = quote! {
                        byte_index = _bit_sum / 8;
                        end_byte_index = byte_index + #total_size;
                        if end_byte_index > bytes.len() {
                            return Err(::bebytes::BeBytesError::InsufficientData {
                                expected: end_byte_index,
                                actual: bytes.len(),
                            });
                        }
                        let #field_name = match bytes[byte_index] {
                            0x00 => None,
                            0x01 => {
                                let value_start = byte_index + 1;
                                #value_parsing
                            }
                            tag => return Err(::bebytes::BeBytesError::InvalidDiscriminant {
                                value: tag,
                                type_name: "Option",
                            }),
                        };
                        _bit_sum += 8 * #total_size;
                    };

                    let value_writing =
                        create_option_inner_writing(inner_tp, processing_ctx.endianness);

                    let writing = quote! {
                        match #field_name {
                            None => {
                                bytes.reserve(#total_size);
                                ::bebytes::BufMut::put_u8(bytes, 0x00);
                                bytes.extend_from_slice(&[0u8; #field_size]);
                            }
                            Some(inner_val) => {
                                bytes.reserve(#total_size);
                                ::bebytes::BufMut::put_u8(bytes, 0x01);
                                #value_writing
                            }
                        }
                        _bit_sum += #total_size * 8;
                    };

                    let direct_writing =
                        crate::functional::pure_helpers::create_option_primitive_direct_writing(
                            field_name,
                            &inner_type,
                            field_size,
                            processing_ctx.endianness,
                        );

                    return Ok(crate::functional::FieldProcessResult::new(
                        quote! {},
                        parsing,
                        writing,
                        direct_writing,
                        accessor,
                        bit_sum,
                    ));
                }
            }

            if let syn::Type::Array(arr) = &inner_type {
                return process_option_array(field_name, arr);
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

    let accessor = create_field_accessor(field_name, AccessorMode::Clone);

    let bit_sum = quote! {
        bit_sum += 8 * #field_type::field_size();
    };

    let try_from_bytes_method = utils::get_try_from_bytes_method(processing_ctx.endianness);
    let to_bytes_method = utils::get_to_bytes_method(processing_ctx.endianness);
    let encode_method = utils::get_encode_to_method(processing_ctx.endianness);

    let parsing = quote_spanned! { context.field.span() =>
        byte_index = _bit_sum / 8;
        let predicted_size = #field_type::field_size();
        end_byte_index = usize::min(bytes.len(), byte_index + predicted_size);
        let (#field_name, bytes_read) = #field_type::#try_from_bytes_method(&bytes[byte_index..end_byte_index])?;
        _bit_sum += bytes_read * 8;
    };

    let writing = quote_spanned! { context.field.span() =>
        let bytes_data = &BeBytes::#to_bytes_method(&#field_name);
        bytes.reserve(bytes_data.len());
        bytes.extend_from_slice(bytes_data);
        _bit_sum += bytes_data.len() * 8;
    };

    let direct_writing = quote_spanned! { context.field.span() =>
        let _ = BeBytes::#encode_method(&#field_name, buf);
    };
    crate::functional::FieldProcessResult::new(
        quote! {},
        parsing,
        writing,
        direct_writing,
        accessor,
        bit_sum,
    )
}

// Functional version for String fields
fn process_string_functional(
    context: &FieldContext,
    size: Option<usize>,
    string_size_ident: Option<Vec<syn::Ident>>,
    _processing_ctx: &crate::functional::ProcessingContext,
) -> Result<crate::functional::FieldProcessResult, syn::Error> {
    let field_name = &context.field_name;
    let field = context.field;
    let is_last_field = context.is_last_field;

    let accessor = create_field_accessor(field_name, AccessorMode::Reference);

    // Generate parsing code based on size constraints
    let (bit_sum, parsing, writing) = match (size, string_size_ident) {
        // Fixed size from attribute: #[With(size(N))]
        (Some(s), None) => generate_fixed_size_string(field_name, s),
        // Size from field: #[FromField(field_name)]
        (_, Some(ident_path)) => generate_field_size_string(field_name, &ident_path),
        // Unbounded (last field only)
        (None, None) => {
            if !is_last_field {
                return Err(syn::Error::new(
                    field.ty.span(),
                    "Unbounded strings can only be used as the last field of a struct",
                ));
            }
            generate_unbounded_string(field_name)
        }
    };

    let direct_writing = quote! {
        let string_bytes = <::bebytes::Utf8 as ::bebytes::StringInterpreter>::to_bytes(&#field_name);
        buf.put_slice(string_bytes);
    };
    Ok(crate::functional::FieldProcessResult::new(
        quote! {},
        parsing,
        writing,
        direct_writing,
        accessor,
        bit_sum,
    ))
}

// Functional version for Size Expression fields
fn process_size_expression_functional(
    context: &FieldContext,
    size_expr: &crate::size_expr::SizeExpression,
    _processing_ctx: &crate::functional::ProcessingContext,
) -> Result<crate::functional::FieldProcessResult, syn::Error> {
    let field_name = &context.field_name;
    let field_type = context.field_type;

    let accessor = create_field_accessor(field_name, AccessorMode::Clone);

    // Generate the size calculation code
    let size_calculation = size_expr.generate_evaluation_code();

    // Generate parsing and writing code based on field type
    match field_type {
        syn::Type::Path(tp) if !tp.path.segments.is_empty() => {
            let segment = &tp.path.segments[0];
            match &segment.ident {
                ident if ident == "Vec" => {
                    // Generate Vec<u8> parsing and writing
                    let (bit_sum, parsing, writing) =
                        generate_size_expression_vector(field_name, &size_calculation);
                    let direct_writing = quote! {
                        buf.put_slice(&#field_name);
                    };
                    Ok(crate::functional::FieldProcessResult::new(
                        quote! {},
                        parsing,
                        writing,
                        direct_writing,
                        accessor,
                        bit_sum,
                    ))
                }
                ident if ident == "String" => {
                    // Generate String parsing and writing
                    let (bit_sum, parsing, writing) =
                        generate_size_expression_string(field_name, &size_calculation);
                    let direct_writing = quote! {
                        let string_bytes = <::bebytes::Utf8 as ::bebytes::StringInterpreter>::to_bytes(&#field_name);
                        buf.put_slice(string_bytes);
                    };
                    Ok(crate::functional::FieldProcessResult::new(
                        quote! {},
                        parsing,
                        writing,
                        direct_writing,
                        accessor,
                        bit_sum,
                    ))
                }
                _ => Err(syn::Error::new_spanned(
                    field_type,
                    "Size expressions are only supported for Vec<u8> and String types",
                )),
            }
        }
        _ => Err(syn::Error::new_spanned(
            field_type,
            "Size expressions are only supported for Vec<u8> and String types",
        )),
    }
}

// Generate code for fixed-size strings
fn generate_fixed_size_string(
    field_name: &syn::Ident,
    size: usize,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let bit_sum = crate::functional::pure_helpers::create_byte_bit_sum(size);

    let parsing = quote! {
        byte_index = _bit_sum / 8;
        let end_index = byte_index + #size;
        if end_index > bytes.len() {
            return Err(::bebytes::BeBytesError::InsufficientData {
                expected: end_index,
                actual: bytes.len(),
            });
        }
        let string_bytes = &bytes[byte_index..end_index];
        let #field_name = <::bebytes::Utf8 as ::bebytes::StringInterpreter>::from_bytes(string_bytes)?;
        _bit_sum += #size * 8;
    };

    let writing = quote! {
        let string_bytes = <::bebytes::Utf8 as ::bebytes::StringInterpreter>::to_bytes(&#field_name);
        if string_bytes.len() != #size {
            panic!(
                "String field {} has length {} but expected fixed size {}",
                stringify!(#field_name),
                string_bytes.len(),
                #size
            );
        }
        bytes.reserve(#size);
        bytes.extend_from_slice(string_bytes);
        _bit_sum += #size * 8;
    };

    (bit_sum, parsing, writing)
}

fn generate_field_size_string(
    field_name: &syn::Ident,
    ident_path: &[proc_macro2::Ident],
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let bit_sum = crate::functional::pure_helpers::create_byte_bit_sum(0); // Variable size doesn't contribute to bit sum

    let field_access = crate::functional::pure_helpers::generate_field_access_path(ident_path);

    let parsing = quote! {
        byte_index = _bit_sum / 8;
        let string_size = (#field_access) as usize;
        let end_index = byte_index + string_size;
        if end_index > bytes.len() {
            return Err(::bebytes::BeBytesError::InsufficientData {
                expected: end_index,
                actual: bytes.len(),
            });
        }
        let string_bytes = &bytes[byte_index..end_index];
        let #field_name = <::bebytes::Utf8 as ::bebytes::StringInterpreter>::from_bytes(string_bytes)?;
        _bit_sum += string_size * 8;
    };

    let writing = quote! {
        let string_bytes = <::bebytes::Utf8 as ::bebytes::StringInterpreter>::to_bytes(&#field_name);
        bytes.reserve(string_bytes.len());
        bytes.extend_from_slice(string_bytes);
        _bit_sum += string_bytes.len() * 8;
    };

    (bit_sum, parsing, writing)
}

fn generate_unbounded_string(
    field_name: &syn::Ident,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let bit_sum = crate::functional::pure_helpers::create_byte_bit_sum(0); // Unbounded doesn't contribute to bit sum

    let parsing = quote! {
        byte_index = _bit_sum / 8;
        let remaining_bytes = &bytes[byte_index..];
        let #field_name = <::bebytes::Utf8 as ::bebytes::StringInterpreter>::from_bytes(remaining_bytes)?;
        _bit_sum += remaining_bytes.len() * 8;
    };

    let writing = quote! {
        let string_bytes = <::bebytes::Utf8 as ::bebytes::StringInterpreter>::to_bytes(&#field_name);
        bytes.reserve(string_bytes.len());
        bytes.extend_from_slice(string_bytes);
        _bit_sum += string_bytes.len() * 8;
    };

    (bit_sum, parsing, writing)
}

// Generate code for Vec<u8> with size expressions
fn generate_size_expression_vector(
    field_name: &syn::Ident,
    size_calculation: &proc_macro2::TokenStream,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let bit_sum = crate::functional::pure_helpers::create_byte_bit_sum(0); // Variable size doesn't contribute to bit sum

    let parsing = quote! {
        byte_index = _bit_sum / 8;
        let field_size = #size_calculation;
        if bytes.len() < byte_index + field_size {
            return Err(::bebytes::BeBytesError::InsufficientData {
                expected: byte_index + field_size,
                actual: bytes.len(),
            });
        }
        let #field_name = bytes[byte_index..byte_index + field_size].to_vec();
        _bit_sum += field_size * 8;
    };

    let writing = quote! {
        let field_size = #size_calculation;
        if #field_name.len() != field_size {
            panic!("Vector size {} does not match expected size {}", #field_name.len(), field_size);
        }
        bytes.extend_from_slice(&#field_name);
        _bit_sum += #field_name.len() * 8;
    };

    (bit_sum, parsing, writing)
}

// Generate code for String with size expressions
fn generate_size_expression_string(
    field_name: &syn::Ident,
    size_calculation: &proc_macro2::TokenStream,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let bit_sum = crate::functional::pure_helpers::create_byte_bit_sum(0); // Variable size doesn't contribute to bit sum

    let parsing = quote! {
        byte_index = _bit_sum / 8;
        let field_size = #size_calculation;
        if bytes.len() < byte_index + field_size {
            return Err(::bebytes::BeBytesError::InsufficientData {
                expected: byte_index + field_size,
                actual: bytes.len(),
            });
        }
        let string_bytes = &bytes[byte_index..byte_index + field_size];
        let #field_name = <::bebytes::Utf8 as ::bebytes::StringInterpreter>::from_bytes(string_bytes)?;
        _bit_sum += field_size * 8;
    };

    let writing = quote! {
        let string_bytes = <::bebytes::Utf8 as ::bebytes::StringInterpreter>::to_bytes(&#field_name);
        let field_size = #size_calculation;
        if string_bytes.len() != field_size {
            panic!("String size {} does not match expected size {}", string_bytes.len(), field_size);
        }
        bytes.extend_from_slice(string_bytes);
        _bit_sum += string_bytes.len() * 8;
    };

    (bit_sum, parsing, writing)
}

fn process_until_marker_functional(
    context: &FieldContext,
    marker: u8,
    _processing_ctx: &crate::functional::ProcessingContext,
) -> crate::functional::FieldProcessResult {
    let field_name = &context.field_name;
    let is_last_field = context.is_last_field;

    // For UntilMarker, Vec<u8> is the only supported type for now
    // Since UntilMarker reads bytes until the marker

    let accessor = quote! { let #field_name = &self.#field_name; };

    // Bit sum is dynamic for marker fields
    let bit_sum = quote! {};

    let parsing = if is_last_field {
        // Last field: consume all remaining bytes if marker not found
        quote! {
            byte_index = _bit_sum / 8;
            let mut #field_name = Vec::new();

            // Read bytes until we find the marker
            while byte_index < bytes.len() && bytes[byte_index] != #marker {
                #field_name.push(bytes[byte_index]);
                byte_index += 1;
                _bit_sum += 8;
            }

            // Skip the marker byte if present
            if byte_index < bytes.len() && bytes[byte_index] == #marker {
                byte_index += 1;
                _bit_sum += 8;
            }
        }
    } else {
        // Not last field: error if marker not found
        quote! {
            byte_index = _bit_sum / 8;
            let mut #field_name = Vec::new();
            let mut marker_found = false;

            // Read bytes until we find the marker
            while byte_index < bytes.len() {
                if bytes[byte_index] == #marker {
                    marker_found = true;
                    byte_index += 1;
                    _bit_sum += 8;
                    break;
                }
                #field_name.push(bytes[byte_index]);
                byte_index += 1;
                _bit_sum += 8;
            }

            // Error if marker not found and this isn't the last field
            if !marker_found {
                return Err(::bebytes::BeBytesError::MarkerNotFound {
                    marker: #marker,
                    field: stringify!(#field_name),
                });
            }
        }
    };

    let writing = quote! {
        bytes.extend_from_slice(&#field_name);
        _bit_sum += #field_name.len() * 8;
        ::bebytes::BufMut::put_u8(bytes, #marker);
        _bit_sum += 8;
    };

    let direct_writing = quote! {
        buf.put_slice(&#field_name);
        buf.put_u8(#marker);
    };

    crate::functional::FieldProcessResult::new(
        quote! {},
        parsing,
        writing,
        direct_writing,
        accessor,
        bit_sum,
    )
}

fn process_after_marker_functional(
    context: &FieldContext,
    marker: u8,
    _processing_ctx: &crate::functional::ProcessingContext,
) -> crate::functional::FieldProcessResult {
    let field_name = &context.field_name;

    let accessor = quote! { let #field_name = &self.#field_name; };

    // Bit sum is dynamic for marker fields
    let bit_sum = quote! {};

    let parsing = quote! {
        byte_index = _bit_sum / 8;

        // Find the marker byte
        let marker_pos = bytes[byte_index..].iter().position(|&b| b == #marker);

        let #field_name = if let Some(pos) = marker_pos {
            // Skip past the marker
            byte_index += pos + 1;
            _bit_sum += (pos + 1) * 8;

            // Read all remaining bytes
            let remaining = bytes[byte_index..].to_vec();
            _bit_sum += remaining.len() * 8;
            remaining
        } else {
            // No marker found, field is empty
            Vec::new()
        };
    };

    let writing = quote! {
        ::bebytes::BufMut::put_u8(bytes, #marker);
        _bit_sum += 8;
        bytes.extend_from_slice(&#field_name);
        _bit_sum += #field_name.len() * 8;
    };

    let direct_writing = quote! {
        buf.put_u8(#marker);
        buf.put_slice(&#field_name);
    };

    crate::functional::FieldProcessResult::new(
        quote! {},
        parsing,
        writing,
        direct_writing,
        accessor,
        bit_sum,
    )
}

fn process_vec_of_vecs_with_marker_functional(
    context: &FieldContext,
    size: Option<usize>,
    field_path: Option<Vec<syn::Ident>>,
    marker: u8,
    _processing_ctx: &crate::functional::ProcessingContext,
) -> crate::functional::FieldProcessResult {
    let field_name = &context.field_name;

    let accessor = quote! { let #field_name = &self.#field_name; };

    // Bit sum is dynamic for marker fields
    let bit_sum = quote! {};

    // Generate size access code
    let size_expr = if let Some(size) = size {
        quote! { #size }
    } else if let Some(field_path) = field_path {
        let field_access = field_path.iter().fold(quote! {}, |acc, ident| {
            if acc.is_empty() {
                quote! { #ident }
            } else {
                quote! { #acc.#ident }
            }
        });
        quote! { #field_access as usize }
    } else {
        // This should never happen due to validation in determine_field_type
        quote! { 0 }
    };

    let parsing = quote! {
        byte_index = _bit_sum / 8;
        let mut #field_name = Vec::new();

        // Read exactly the specified number of segments
        for segment_idx in 0..#size_expr {
            let mut current_section = Vec::new();
            let mut marker_found = false;

            // Read until marker for this segment
            while byte_index < bytes.len() {
                if bytes[byte_index] == #marker {
                    marker_found = true;
                    byte_index += 1;
                    _bit_sum += 8;
                    break;
                }
                current_section.push(bytes[byte_index]);
                byte_index += 1;
                _bit_sum += 8;
            }

            // Error if marker not found for any segment
            if !marker_found {
                return Err(::bebytes::BeBytesError::MarkerNotFound {
                    marker: #marker,
                    field: stringify!(#field_name),
                });
            }

            #field_name.push(current_section);
        }
    };

    let writing = quote! {
        for section in #field_name {
            bytes.extend_from_slice(section);
            _bit_sum += section.len() * 8;
            ::bebytes::BufMut::put_u8(bytes, #marker);
            _bit_sum += 8;
        }
    };

    let direct_writing = quote! {
        for section in #field_name {
            buf.put_slice(section);
            buf.put_u8(#marker);
        }
    };

    crate::functional::FieldProcessResult::new(
        quote! {},
        parsing,
        writing,
        direct_writing,
        accessor,
        bit_sum,
    )
}
