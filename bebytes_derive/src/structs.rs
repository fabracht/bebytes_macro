use crate::*;
use quote::{__private::Span, quote, quote_spanned};
use syn::spanned::Spanned;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

enum FieldType {
    U8Field(usize, usize), // size, position
    PrimitiveType,
    Array(usize),                              // array_length
    Vector(Option<usize>, Option<syn::Ident>), // size, vec_size_ident
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
    field_limit_check: Vec<proc_macro2::TokenStream>,
    errors: Vec<proc_macro2::TokenStream>,
    field_parsing: Vec<proc_macro2::TokenStream>,
    pub bit_sum: Vec<proc_macro2::TokenStream>,
    field_writing: Vec<proc_macro2::TokenStream>,
    pub named_fields: Vec<proc_macro2::TokenStream>,
    total_size: usize,
}

pub struct StructContext<'a> {
    pub field_limit_check: &'a mut Vec<proc_macro2::TokenStream>,
    pub errors: &'a mut Vec<proc_macro2::TokenStream>,
    pub field_parsing: &'a mut Vec<proc_macro2::TokenStream>,
    pub bit_sum: &'a mut Vec<proc_macro2::TokenStream>,
    pub field_writing: &'a mut Vec<proc_macro2::TokenStream>,
    pub named_fields: &'a mut Vec<proc_macro2::TokenStream>,
    pub fields: &'a syn::FieldsNamed,
    pub total_size: usize,
    pub last_field: Option<&'a syn::Field>,
    pub endianness: crate::consts::Endianness,
}

fn push_field_accessor(field_data: &mut FieldData, field_name: &syn::Ident, needs_owned: bool) {
    let accessor = if needs_owned {
        quote! { let #field_name = self.#field_name.to_owned(); }
    } else {
        quote! { let #field_name = self.#field_name; }
    };
    field_data.named_fields.push(accessor);
}

fn push_bit_sum(field_data: &mut FieldData, size: usize) {
    field_data.bit_sum.push(quote! { bit_sum += #size * 8; });
}

fn push_byte_indices(tokens: &mut Vec<proc_macro2::TokenStream>, field_size: usize) {
    tokens.push(quote! {
        byte_index = _bit_sum / 8;
        end_byte_index = byte_index + #field_size;
        _bit_sum += 8 * #field_size;
    });
}

fn handle_u8_field(
    context: &FieldContext,
    size: usize,
    pos: usize,
    field_data: &mut FieldData,
    endianness: crate::consts::Endianness,
) {
    let FieldContext {
        field_name,
        field_type,
        field,
        ..
    } = context;

    push_field_accessor(field_data, field_name, false);

    let number_length = utils::get_number_size(field_type, field, &mut field_data.errors)
        .unwrap_or_else(|| {
            let error = syn::Error::new(field_type.span(), "Type not supported");
            field_data.errors.push(error.to_compile_error());
            0
        });

    field_data.bit_sum.push(quote! {bit_sum += #size;});

    let mask: u128 = (1 << size) - 1;
    field_data.field_limit_check.push(quote! {
        if #field_name > #mask as #field_type {
            panic!("Value of field {} is out of range (max value: {})",
                stringify!(#field_name),
                #mask
            );
        }
    });

    let from_bytes_method = utils::get_from_bytes_method(endianness);
    let to_bytes_method = utils::get_to_bytes_method(endianness);
    let bit_shift_direction = utils::get_u8_bit_shift_direction(size, pos, endianness);
    let bit_write_shift = utils::get_u8_bit_write_shift(size, pos, endianness);

    if number_length > 1 {
        let chunks =
            utils::generate_chunks(number_length, syn::Ident::new("chunk", Span::call_site()));

        field_data.field_parsing.push(quote! {
            let mut inner_total_size = #pos;
            let mut #field_name = 0 as #field_type;
            bytes.chunks(#number_length).for_each(|chunk| {
                let u_type = #field_type::#from_bytes_method(#chunks);
                let shift_left = _bit_sum % 8;
                let left_shifted_u_type = u_type << shift_left;
                let shift_right = 8 * #number_length - #size;
                let shifted_u_type = left_shifted_u_type >> shift_right;
                #field_name = shifted_u_type & #mask as #field_type;
            });
            _bit_sum += #size;
        });

        field_data.field_writing.push(quote! {
            if #field_name > #mask as #field_type {
                panic!(
                    "Value {} for field {} exceeds the maximum allowed value {}.",
                    #field_name,
                    stringify!(#field_name),
                    #mask
                );
            }
            let masked_value = #field_name & #mask as #field_type;
            let shift_left = (#number_length * 8) - #size;
            let shift_right = (#pos % 8);
            let shifted_masked_value = (masked_value << shift_left) >> shift_right;
            let byte_values = #field_type::#to_bytes_method(shifted_masked_value);
            for i in 0..#number_length {
                if bytes.len() <= _bit_sum / 8 + i {
                    bytes.resize(_bit_sum / 8 + i + 1, 0);
                }
                bytes[_bit_sum / 8 + i] |= byte_values[i];
            }
            _bit_sum += #size;
        });
    } else {
        field_data.field_parsing.push(quote! {
            let #field_name = (bytes[_bit_sum / 8] as #field_type >> (#bit_shift_direction as usize) as #field_type) & (#mask as #field_type);
            _bit_sum += #size;
        });

        field_data.field_writing.push(quote! {
            if #field_name > #mask as #field_type {
                panic!(
                    "Value {} for field {} exceeds the maximum allowed value {}.",
                    #field_name,
                    stringify!(#field_name),
                    #mask
                );
            }
            if bytes.len() <= _bit_sum / 8 {
                bytes.resize(_bit_sum / 8 + 1, 0);
            }
            bytes[_bit_sum / 8] |= (#field_name as u8) << (#bit_write_shift as usize);
            _bit_sum += #size;
        });
    }

    field_data.total_size += size;
}

fn handle_primitive_type(
    context: &FieldContext,
    field_data: &mut FieldData,
    endianness: crate::consts::Endianness,
) {
    let FieldContext {
        field_name,
        field_type,
        field,
        ..
    } = context;

    push_field_accessor(field_data, field_name, false);

    let field_size = match utils::get_number_size(field_type, field, &mut field_data.errors) {
        Some(value) => value,
        None => return,
    };

    push_bit_sum(field_data, field_size);

    let mut parsing = Vec::new();
    push_byte_indices(&mut parsing, field_size);

    let from_bytes_method = utils::get_from_bytes_method(endianness);
    let to_bytes_method = utils::get_to_bytes_method(endianness);

    parsing.push(quote! {
        let #field_name = <#field_type>::#from_bytes_method({
            let slice = &bytes[byte_index..end_byte_index];
            let mut arr = [0; #field_size];
            arr.copy_from_slice(slice);
            arr
        });
    });
    field_data.field_parsing.extend(parsing);

    field_data.field_writing.push(quote! {
        let field_slice = &#field_name.#to_bytes_method();
        bytes.extend_from_slice(field_slice);
        _bit_sum += field_slice.len() * 8;
    });
}

fn handle_array(context: &FieldContext, length: usize, field_data: &mut FieldData) {
    let FieldContext {
        field_name,
        field_type,
        ..
    } = context;

    if let syn::Type::Array(tp) = field_type {
        if let syn::Type::Path(elem) = &*tp.elem {
            let segments = &elem.path.segments;
            if segments.len() == 1 && segments[0].ident == "u8" {
                push_field_accessor(field_data, field_name, true);
                push_bit_sum(field_data, length);

                field_data.field_parsing.push(quote! {
                    byte_index = _bit_sum / 8;
                    let mut #field_name = [0u8; #length];
                    #field_name.copy_from_slice(&bytes[byte_index..#length + byte_index]);
                    _bit_sum += 8 * #length;
                });

                field_data.field_writing.push(quote! {
                    bytes.extend_from_slice(&#field_name);
                    _bit_sum += #length * 8;
                });
            }
        }
    }
}

fn handle_vector(
    context: &FieldContext,
    size: Option<usize>,
    vec_size_ident: Option<syn::Ident>,
    field_data: &mut FieldData,
    endianness: crate::consts::Endianness,
) {
    let FieldContext {
        field_name,
        field_type,
        field,
        is_last_field,
    } = context;

    push_field_accessor(field_data, field_name, true);

    if let syn::Type::Path(tp) = field_type {
        if let Some(syn::Type::Path(ref inner_tp)) = utils::solve_for_inner_type(tp, "Vec") {
            // Handle vector of primitive types (u8, etc.)

            if utils::is_primitive_type(inner_tp) {
                match (size, vec_size_ident.clone()) {
                    (_, Some(ident)) => {
                        field_data.bit_sum.push(quote! { bit_sum = 4096 * 8; });
                        field_data.field_parsing.push(quote! {
                            let vec_length = #ident as usize;
                            byte_index = _bit_sum / 8;
                            let end_index = byte_index + vec_length;
                            if end_index > bytes.len() {
                                panic!("Not enough bytes to parse a vector of size {}", vec_length);
                            }
                            let #field_name = Vec::from(&bytes[byte_index..end_index]);
                            _bit_sum += vec_length * 8;
                        });
                    }
                    (Some(s), None) => {
                        push_bit_sum(field_data, s);
                        field_data.field_parsing.push(quote! {
                            let vec_length = #s as usize;
                            byte_index = _bit_sum / 8;
                            let end_index = byte_index + vec_length;
                            let #field_name = Vec::from(&bytes[byte_index..end_index]);
                            _bit_sum += #s * 8;
                        });
                    }
                    (None, None) => {
                        if !is_last_field {
                            let error = syn::Error::new(
                                    field.ty.span(),
                                    "Unbounded vectors can only be used as padding at the end of a struct",
                                );
                            field_data.errors.push(error.to_compile_error());
                        }
                        field_data.bit_sum.push(quote! { bit_sum = 4096 * 8; });
                        field_data.field_parsing.push(quote! {
                            byte_index = _bit_sum / 8;
                            let #field_name = Vec::from(&bytes[byte_index..]);
                            _bit_sum += #field_name.len() * 8;
                        });
                    }
                }

                field_data.field_writing.push(quote! {
                    bytes.extend_from_slice(&#field_name);
                    _bit_sum += #field_name.len() * 8;
                });
            } else {
                // Handle vector of custom types (Vec<CustomType>)
                let inner_type_path = &inner_tp.path;
                let inner_type_name = quote! { #inner_type_path };

                // Get the appropriate endianness methods
                let try_from_bytes_method = utils::get_try_from_bytes_method(endianness);
                let to_bytes_method = utils::get_to_bytes_method(endianness);

                // Initialize an empty vector
                field_data.field_parsing.push(quote! {
                    let mut #field_name = Vec::new();
                });

                if !is_last_field {
                    // For non-last fields, we need size information
                    if let Some(vec_size) = size {
                        field_data.field_parsing.push(quote! {
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
                            });
                    } else if let Some(ident) = vec_size_ident {
                        field_data.field_parsing.push(quote! {
                                let vec_length = #ident as usize;
                                let mut bytes_consumed = 0;
                                for _ in 0..vec_length {
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
                            });
                    } else {
                        let error = syn::Error::new(
                                field.ty.span(),
                                "Vectors of custom types need size information. Use #[With(size(n))] or #[FromField(field_name)]",
                            );
                        field_data.errors.push(error.to_compile_error());
                    }
                } else {
                    // For the last field, we can consume all remaining bytes
                    field_data.field_parsing.push(quote! {
                            let mut bytes_consumed = 0;
                            while _bit_sum / 8 + bytes_consumed < bytes.len() {
                                match #inner_type_name::#try_from_bytes_method(&bytes[_bit_sum / 8 + bytes_consumed..]) {
                                    Ok((item, consumed)) => {
                                        #field_name.push(item);
                                        bytes_consumed += consumed;
                                    }
                                    Err(e) => break, // Stop on error
                                }
                            }
                            _bit_sum += bytes_consumed * 8;
                        });
                }

                // Serialize the vector
                field_data.field_writing.push(quote! {
                    for item in &#field_name {
                        let item_bytes = BeBytes::#to_bytes_method(item);
                        bytes.extend_from_slice(&item_bytes);
                        _bit_sum += item_bytes.len() * 8;
                    }
                });
            }
        }
    }
}

fn handle_option_type(
    context: &FieldContext,
    field_data: &mut FieldData,
    endianness: crate::consts::Endianness,
) {
    let FieldContext {
        field_name,
        field_type,
        field,
        ..
    } = context;

    push_field_accessor(field_data, field_name, false);

    if let syn::Type::Path(tp) = field_type {
        if let Some(inner_type) = utils::solve_for_inner_type(tp, "Option") {
            if let syn::Type::Path(inner_tp) = &inner_type {
                if utils::is_primitive_type(inner_tp) {
                    let field_size =
                        match utils::get_number_size(&inner_type, field, &mut field_data.errors) {
                            Some(value) => value,
                            None => return,
                        };

                    push_bit_sum(field_data, field_size);

                    let from_bytes_method = utils::get_from_bytes_method(endianness);
                    let to_bytes_method = utils::get_to_bytes_method(endianness);

                    let mut parsing = Vec::new();
                    push_byte_indices(&mut parsing, field_size);
                    parsing.push(quote! {
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
                    });
                    field_data.field_parsing.extend(parsing);

                    field_data.field_writing.push(quote! {
                        let bytes_data = &#field_name.unwrap_or(0).#to_bytes_method();
                        bytes.extend_from_slice(bytes_data);
                        _bit_sum += bytes_data.len() * 8;
                    });
                }
            }
        }
    }
}

fn handle_custom_type(
    context: &FieldContext,
    field_data: &mut FieldData,
    endianness: crate::consts::Endianness,
) {
    let FieldContext {
        field_name,
        field_type,
        field,
        ..
    } = context;

    let needs_owned = !utils::is_copy(field_type);
    push_field_accessor(field_data, field_name, needs_owned);

    field_data.bit_sum.push(quote! {
        bit_sum += 8 * #field_type::field_size();
    });

    let try_from_bytes_method = utils::get_try_from_bytes_method(endianness);
    let to_bytes_method = utils::get_to_bytes_method(endianness);

    field_data.field_parsing.push(quote_spanned! { field.span() =>
        byte_index = _bit_sum / 8;
        let predicted_size = #field_type::field_size();
        end_byte_index = usize::min(bytes.len(), byte_index + predicted_size);
        let (#field_name, bytes_read) = #field_type::#try_from_bytes_method(&bytes[byte_index..end_byte_index])?;
        _bit_sum += bytes_read * 8;
    });

    field_data
        .field_writing
        .push(quote_spanned! { field.span() =>
            let bytes_data = &BeBytes::#to_bytes_method(&#field_name);
            bytes.extend_from_slice(bytes_data);
            _bit_sum += bytes_data.len() * 8;
        });
}

fn determine_field_type(
    context: &FieldContext,
    attrs: &[syn::Attribute],
    errors: &mut Vec<proc_macro2::TokenStream>,
) -> Option<FieldType> {
    let mut u8_attribute_present = false;
    let (pos, size, vec_size_ident) =
        attrs::parse_attributes(attrs.to_vec(), &mut u8_attribute_present, errors);

    if u8_attribute_present {
        if let syn::Type::Path(tp) = context.field_type {
            if !utils::is_supported_primitive_type(tp) {
                let error = syn::Error::new(
                    context.field_type.span(),
                    "Unsupported type for U8 attribute",
                );
                errors.push(error.to_compile_error());
                return None;
            }
        }
        if let (Some(pos), Some(size)) = (pos, size) {
            return Some(FieldType::U8Field(size, pos));
        }
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
                ident if !utils::is_primitive_identity(ident) => Some(FieldType::CustomType),
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn handle_struct(context: StructContext) {
    // First validate bit ranges
    if let Err(validation_error) = crate::bit_validation::validate_field_sequence(context.fields) {
        context.errors.push(validation_error);
        return;
    }

    let mut field_data = FieldData {
        field_limit_check: Vec::new(),
        errors: Vec::new(),
        field_parsing: Vec::new(),
        bit_sum: Vec::new(),
        field_writing: Vec::new(),
        named_fields: Vec::new(),
        total_size: context.total_size,
    };

    for field in context.fields.named.clone() {
        let is_last = context
            .last_field
            .map(|last| last.ident == field.ident)
            .unwrap_or(false);

        let field_context = FieldContext {
            field: &field,
            field_name: field.ident.clone().unwrap(),
            field_type: &field.ty,
            is_last_field: is_last,
        };

        match determine_field_type(&field_context, &field.attrs, &mut field_data.errors) {
            Some(field_type) => match field_type {
                FieldType::U8Field(size, pos) => handle_u8_field(
                    &field_context,
                    size,
                    pos,
                    &mut field_data,
                    context.endianness,
                ),
                FieldType::PrimitiveType => {
                    handle_primitive_type(&field_context, &mut field_data, context.endianness)
                }
                FieldType::Array(length) => handle_array(&field_context, length, &mut field_data),
                FieldType::Vector(size, vec_size_ident) => handle_vector(
                    &field_context,
                    size,
                    vec_size_ident,
                    &mut field_data,
                    context.endianness,
                ),
                FieldType::OptionType => {
                    handle_option_type(&field_context, &mut field_data, context.endianness)
                }
                FieldType::CustomType => {
                    handle_custom_type(&field_context, &mut field_data, context.endianness)
                }
            },
            None => continue,
        }
    }

    *context.field_limit_check = field_data.field_limit_check;
    *context.errors = field_data.errors;
    *context.field_parsing = field_data.field_parsing;
    *context.bit_sum = field_data.bit_sum;
    *context.field_writing = field_data.field_writing;
    *context.named_fields = field_data.named_fields;
}
