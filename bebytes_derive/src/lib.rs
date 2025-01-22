#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

use proc_macro::TokenStream;
use quote::{__private::Span, quote, quote_spanned};
use syn::{
    parenthesized, parse_macro_input, spanned::Spanned, AngleBracketedGenericArguments, Data,
    DeriveInput, Fields, LitInt,
};

#[cfg(feature = "std")]
use std::{string::ToString, vec::Vec};

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

use core::fmt::Write;

const PRIMITIVES: [&str; 17] = [
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
    "f64", "bool", "char", "str",
];
const SUPPORTED_PRIMITIVES: [&str; 10] = [
    "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128",
];

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

                let mut total_size: usize = 0;
                let last_field = fields.named.last();
                let mut is_last_field = false;

                for field in fields.named.clone().into_iter() {
                    if let Some(last_field) = last_field {
                        is_last_field = last_field.ident == field.ident;
                    }
                    let mut u8_attribute_present = false;
                    let attributes = field.attrs.clone();
                    let field_name = field.ident.clone().unwrap();
                    let field_type = &field.ty;
                    let (pos, mut size, vec_size_ident_option) =
                        parse_attributes(attributes, &mut u8_attribute_present, &mut errors);

                    if u8_attribute_present {
                        match field_type {
                            syn::Type::Path(tp) if !is_supported_primitive_type(tp) => {
                                let error = syn::Error::new(
                                    field_type.span(),
                                    "Unsupported type for U8 attribute",
                                );
                                errors.push(error.to_compile_error());
                                break;
                            }
                            _ => {}
                        }
                        named_fields.push(quote! { let #field_name = self.#field_name; });
                        let number_length = get_number_size(field_type, &field, &mut errors)
                            .unwrap_or_else(|| {
                                let error =
                                    syn::Error::new(field_type.span(), "Type not supported'");
                                errors.push(error.to_compile_error());
                                0
                            });
                        if pos.is_none() && size.is_none() {
                            let error = syn::Error::new(
                                field.span(),
                                "U8 attribute must have a size and a position",
                            );
                            errors.push(error.to_compile_error());
                        }
                        if let (Some(pos), Some(size)) = (pos, size) {
                            bit_sum.push(quote! {bit_sum += #size;});

                            let mask: u128 = (1 << size) - 1;
                            field_limit_check.push(quote! {
                                if #field_name > #mask as #field_type {
                                    let mut err_msg = String::new();
                                    core::write!(&mut err_msg, "Value of field {} is out of range (max value: {})",
                                        stringify!(#field_name),
                                        #mask
                                    ).unwrap();
                                    panic!("{}", err_msg);
                                }
                            });

                            if pos % 8 != total_size % 8 {
                                let mut message = String::new();
                                core::write!(&mut message, "U8 attributes must obey the sequence specified by the previous attributes. Expected position {} but got {}", total_size, pos).unwrap();
                                errors.push(
                                    syn::Error::new_spanned(&field, message).to_compile_error(),
                                );
                            }
                            if number_length > 1 {
                                let chunks = generate_chunks(
                                    number_length,
                                    syn::Ident::new("chunk", Span::call_site()),
                                );

                                field_parsing.push(quote! {
                                    let mut inner_total_size = #total_size;
                                    let mut #field_name = 0 as #field_type;
                                    bytes.chunks(#number_length).for_each(|chunk| {
                                        let u_type = #field_type::from_be_bytes(#chunks);
                                        let shift_left = _bit_sum % 8;
                                        let left_shifted_u_type = u_type << shift_left;
                                        let shift_right = 8 * #number_length - #size;
                                        let shifted_u_type = left_shifted_u_type >> shift_right;
                                        #field_name = shifted_u_type & #mask as #field_type;
                                    });
                                    _bit_sum += #size;
                                });
                                field_writing.push(quote! {
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
                                    let byte_values = #field_type::to_be_bytes(shifted_masked_value);
                                    for i in 0..#number_length {
                                        if bytes.len() <= _bit_sum / 8 + i {
                                            bytes.resize(_bit_sum / 8 + i + 1, 0);
                                        }
                                        bytes[_bit_sum / 8 + i] |= byte_values[i];
                                    }
                                    _bit_sum += #size;
                                });
                            } else {
                                field_parsing.push(quote! {
                                    let #field_name = (bytes[_bit_sum / 8] as #field_type >> (7 - (#size + #pos % 8 - 1) as #field_type )) & (#mask as #field_type);
                                    _bit_sum += #size;
                                });
                                field_writing.push(quote! {
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
                                    bytes[_bit_sum / 8] |= (#field_name as u8) << (7 - (#size - 1) - #pos % 8 );
                                    _bit_sum += #size;
                                });
                            }
                            total_size += size;
                        }
                    } else {
                        if total_size % 8 != 0 {
                            errors.push(syn::Error::new_spanned(&field, "U8 attributes must add up to 8 before any other non U8 field is added").to_compile_error());
                        }
                        match field_type {
                            syn::Type::Path(tp) if is_primitive_type(tp) => {
                                named_fields.push(quote! { let #field_name = self.#field_name; });
                                let field_size =
                                    match get_number_size(field_type, &field, &mut errors) {
                                        Some(value) => value,
                                        None => continue,
                                    };
                                bit_sum.push(quote! { bit_sum += #field_size * 8;});
                                parse_write_number(
                                    field_size,
                                    &mut field_parsing,
                                    &field_name,
                                    field_type,
                                    &mut field_writing,
                                );
                            }
                            syn::Type::Array(tp) => {
                                named_fields.push(
                                    quote! { let #field_name = self.#field_name.to_owned(); },
                                );

                                let array_length: usize;
                                let len = tp.len.clone();
                                match len {
                                    syn::Expr::Lit(expr_lit) => {
                                        if let syn::Lit::Int(token) = expr_lit.lit {
                                            array_length =
                                                token.base10_parse().unwrap_or_else(|_e| {
                                                    let error = syn::Error::new(
                                                        token.span(),
                                                        "Failed to parse token value",
                                                    );
                                                    errors.push(error.to_compile_error());
                                                    0
                                                });
                                        } else {
                                            let error = syn::Error::new(
                                                field.ty.span(),
                                                "Expected integer type for N",
                                            );
                                            errors.push(error.to_compile_error());
                                            continue;
                                        }
                                    }
                                    _ => {
                                        let error = syn::Error::new(
                                            tp.span(),
                                            "N must be a literal in [T; N]",
                                        );
                                        errors.push(error.to_compile_error());
                                        continue;
                                    }
                                }
                                if let syn::Type::Path(elem) = *tp.elem.clone() {
                                    let syn::TypePath {
                                        path: syn::Path { segments, .. },
                                        ..
                                    } = elem;
                                    match &segments[0] {
                                        syn::PathSegment {
                                            ident,
                                            arguments: syn::PathArguments::None,
                                        } if ident == "u8" => {
                                            bit_sum.push(quote! { bit_sum += 8 * #array_length;});

                                            field_parsing.push(quote! {
                                                byte_index = _bit_sum / 8;
                                                let mut #field_name = [0u8; #array_length];
                                                #field_name.copy_from_slice(&bytes[byte_index..#array_length + byte_index]);
                                                _bit_sum += 8 * #array_length;
                                            });
                                            field_writing.push(quote! {
                                                bytes.extend_from_slice(&#field_name);
                                                _bit_sum += #array_length * 8;
                                            });
                                        }
                                        _ => {
                                            let error = syn::Error::new(
                                                field.ty.span(),
                                                "Unsupported type for [T; N]",
                                            );
                                            errors.push(error.to_compile_error());
                                        }
                                    };
                                }
                            }
                            syn::Type::Path(tp)
                                if !tp.path.segments.is_empty()
                                    && tp.path.segments[0].ident == "Vec" =>
                            {
                                named_fields.push(
                                    quote! { let #field_name = self.#field_name.to_owned(); },
                                );

                                let inner_type = match solve_for_inner_type(tp, "Vec") {
                                    Some(t) => t,
                                    None => {
                                        let error = syn::Error::new(
                                            field.ty.span(),
                                            "Unsupported type for Vec<T>",
                                        );
                                        errors.push(error.to_compile_error());
                                        continue;
                                    }
                                };

                                if let syn::Type::Path(inner_tp) = &inner_type {
                                    if is_primitive_type(inner_tp) {
                                        if let Some(vec_size_ident) = vec_size_ident_option {
                                            bit_sum.push(quote! {
                                                bit_sum = 4096 * 8;
                                            });

                                            field_parsing.push(quote! {
                                                let vec_length = #vec_size_ident as usize;
                                                byte_index = _bit_sum / 8;
                                                let end_index = byte_index + vec_length;
                                                if end_index > bytes.len() {
                                                    let mut error_message = String::new();
                                                    core::write!(&mut error_message, "Not enough bytes to parse a vector of size {}", vec_length).unwrap();
                                                    panic!("{}", error_message);
                                                }
                                                let #field_name = Vec::from(&bytes[byte_index..end_index]);
                                                _bit_sum += vec_length * 8;
                                            });
                                        } else if let Some(s) = size.take() {
                                            bit_sum.push(quote! {
                                                bit_sum += #s * 8;
                                            });
                                            field_parsing.push(quote! {
                                                let vec_length = #s as usize;
                                                byte_index = _bit_sum / 8;
                                                let end_index = byte_index + vec_length;
                                                let #field_name = Vec::from(&bytes[byte_index..end_index]);
                                                _bit_sum += #s * 8;
                                            });
                                        } else {
                                            bit_sum.push(quote! {
                                                bit_sum = 4096 * 8;
                                            });
                                            field_parsing.push(quote! {
                                                byte_index = _bit_sum / 8;
                                                let #field_name = Vec::from(&bytes[byte_index..]);
                                                _bit_sum += #field_name.len() * 8;
                                            });
                                            if !is_last_field {
                                                let error = syn::Error::new(
                                                    field.ty.span(),
                                                    "Unbounded vectors can only be used as padding at the end of a struct",
                                                );
                                                errors.push(error.to_compile_error());
                                            }
                                        }
                                        field_writing.push(quote! {
                                            let field_length = &#field_name.len();
                                            bytes.extend_from_slice(&#field_name);
                                            _bit_sum += #field_name.len() * 8;
                                        });
                                    } else {
                                        let error = syn::Error::new(
                                            inner_type.span(),
                                            "Unsupported type for Vec<T>",
                                        );
                                        errors.push(error.to_compile_error());
                                    }
                                }
                            }
                            syn::Type::Path(tp)
                                if !tp.path.segments.is_empty()
                                    && tp.path.segments[0].ident == "Option" =>
                            {
                                named_fields.push(quote! { let #field_name = self.#field_name; });

                                let inner_type = match solve_for_inner_type(tp, "Option") {
                                    Some(t) => t,
                                    None => {
                                        let error = syn::Error::new(
                                            field.ty.span(),
                                            "Unsupported type for Option<T>",
                                        );
                                        errors.push(error.to_compile_error());
                                        continue;
                                    }
                                };

                                if let syn::Type::Path(inner_tp) = &inner_type {
                                    if is_primitive_type(inner_tp) {
                                        let field_size =
                                            match get_number_size(&inner_type, &field, &mut errors)
                                            {
                                                Some(value) => value,
                                                None => continue,
                                            };
                                        bit_sum.push(quote! { bit_sum += 8 * #field_size;});
                                        field_parsing.push(quote! {
                                            byte_index = _bit_sum / 8;
                                            end_byte_index = byte_index + #field_size;
                                            let #field_name = if bytes[byte_index..end_byte_index] == [0_u8; #field_size] {
                                                None
                                            } else {
                                                _bit_sum += 8 * #field_size;
                                                Some(<#inner_tp>::from_be_bytes({
                                                    let slice = &bytes[byte_index..end_byte_index];
                                                    let mut arr = [0; #field_size];
                                                    arr.copy_from_slice(slice);
                                                    arr
                                                }))
                                            };
                                        });
                                        field_writing.push(quote! {
                                            let be_bytes = &#field_name.unwrap_or(0).to_be_bytes();
                                            bytes.extend_from_slice(be_bytes);
                                            _bit_sum += be_bytes.len() * 8;
                                        });
                                    } else {
                                        let error = syn::Error::new(
                                            inner_type.span(),
                                            "Unsupported type for Option<T>",
                                        );
                                        errors.push(error.to_compile_error());
                                    }
                                }
                            }
                            syn::Type::Path(tp)
                                if !tp.path.segments.is_empty()
                                    && !is_primitive_identity(&tp.path.segments[0].ident) =>
                            {
                                named_fields.push(
                                    quote! { let #field_name = self.#field_name.to_owned(); },
                                );

                                bit_sum.push(quote! { bit_sum += 8 * #field_type::field_size();});
                                field_parsing.push(quote_spanned! { field.span() =>
                                    byte_index = _bit_sum / 8;
                                    let predicted_size = #field_type::field_size();
                                    end_byte_index = usize::min(bytes.len(), byte_index + predicted_size);
                                    let (#field_name, bytes_read) = #field_type::try_from_be_bytes(&bytes[byte_index..end_byte_index])?;
                                    _bit_sum += bytes_read * 8;
                                });
                                field_writing.push(quote_spanned! { field.span() =>
                                    let be_bytes = &BeBytes::to_be_bytes(&#field_name);
                                    bytes.extend_from_slice(be_bytes);
                                    _bit_sum += be_bytes.len() * 8;
                                });
                            }
                            _ => {
                                let mut error_message = String::new();
                                core::write!(
                                    &mut error_message,
                                    "Unsupported type for field {}",
                                    field_name
                                )
                                .unwrap();
                                let error = syn::Error::new(field.ty.span(), error_message);
                                errors.push(error.to_compile_error());
                            }
                        }
                    }
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
                let output = quote! {
                    #error
                };

                output.into()
            }
        },
        Data::Enum(data_enum) => {
            let variants = data_enum.variants;
            let values = variants
                .iter()
                .enumerate()
                .map(|(index, variant)| {
                    let ident = &variant.ident;
                    let mut assigned_value = index as u8;
                    if let Some((_, syn::Expr::Lit(expr_lit))) = &variant.discriminant {
                        if let syn::Lit::Int(token) = &expr_lit.lit {
                            assigned_value = token.base10_parse().unwrap_or_else(|_e| {
                                let error =
                                    syn::Error::new(token.span(), "Failed to parse token value");
                                errors.push(error.to_compile_error());
                                0
                            });
                        }
                    };
                    (ident, assigned_value)
                })
                .collect::<Vec<_>>();

            let from_be_bytes_arms = values
                .iter()
                .map(|(ident, assigned_value)| {
                    quote! {
                        #assigned_value => Ok((Self::#ident, 1)),
                    }
                })
                .collect::<Vec<_>>();

            let to_be_bytes_arms = values
                .iter()
                .map(|(ident, assigned_value)| {
                    quote! {
                        Self::#ident => #assigned_value as u8,
                    }
                })
                .collect::<Vec<_>>();

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

fn parse_write_number(
    field_size: usize,
    field_parsing: &mut Vec<proc_macro2::TokenStream>,
    field_name: &syn::Ident,
    field_type: &syn::Type,
    field_writing: &mut Vec<proc_macro2::TokenStream>,
) {
    field_parsing.push(quote! {
        byte_index = _bit_sum / 8;
        end_byte_index = byte_index + #field_size;
        _bit_sum += 8 * #field_size;
        let #field_name = <#field_type>::from_be_bytes({
            let slice = &bytes[byte_index..end_byte_index];
            let mut arr = [0; #field_size];
            arr.copy_from_slice(slice);
            arr
        });
    });
    field_writing.push(quote! {
        let field_slice = &#field_name.to_be_bytes();
        bytes.extend_from_slice(field_slice);
        _bit_sum += field_slice.len() * 8;
    });
}

fn get_number_size(
    field_type: &syn::Type,
    field: &syn::Field,
    errors: &mut Vec<proc_macro2::TokenStream>,
) -> Option<usize> {
    let field_size = match &field_type {
        syn::Type::Path(tp) if tp.path.is_ident("i8") || tp.path.is_ident("u8") => 1,
        syn::Type::Path(tp) if tp.path.is_ident("i16") || tp.path.is_ident("u16") => 2,
        syn::Type::Path(tp)
            if tp.path.is_ident("i32") || tp.path.is_ident("u32") || tp.path.is_ident("f32") =>
        {
            4
        }
        syn::Type::Path(tp)
            if tp.path.is_ident("i64") || tp.path.is_ident("u64") || tp.path.is_ident("f64") =>
        {
            8
        }
        syn::Type::Path(tp) if tp.path.is_ident("i128") || tp.path.is_ident("u128") => 16,
        _ => {
            let error = syn::Error::new(field.ty.span(), "Unsupported type");
            errors.push(error.to_compile_error());
            return None;
        }
    };
    Some(field_size)
}

fn parse_attributes(
    attributes: Vec<syn::Attribute>,
    u8_attribute_present: &mut bool,
    errors: &mut Vec<proc_macro2::TokenStream>,
) -> (Option<usize>, Option<usize>, Option<proc_macro2::Ident>) {
    let mut pos = None;
    let mut size = None;
    let mut field = None;

    for attr in attributes {
        if attr.path().is_ident("U8") {
            *u8_attribute_present = true;
            if let Err(e) = parse_u8_attribute(&attr, &mut pos, &mut size) {
                errors.push(e.to_compile_error());
            }
        } else if attr.path().is_ident("With") {
            if let Err(e) = parse_with_attribute(&attr, &mut size) {
                errors.push(e.to_compile_error());
            }
        } else if attr.path().is_ident("FromField") {
            if let Err(e) = parse_from_field_attribute(&attr, &mut field) {
                errors.push(e.to_compile_error());
            }
        }
    }

    (pos, size, field)
}

fn parse_u8_attribute(
    attr: &syn::Attribute,
    pos: &mut Option<usize>,
    size: &mut Option<usize>,
) -> Result<(), syn::Error> {
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("pos") {
            let content;
            parenthesized!(content in meta.input);
            let lit: LitInt = content.parse()?;
            let n: usize = lit.base10_parse()?;
            *pos = Some(n);
            Ok(())
        } else if meta.path.is_ident("size") {
            let content;
            parenthesized!(content in meta.input);
            let lit: LitInt = content.parse()?;
            let n: usize = lit.base10_parse()?;
            *size = Some(n);
            Ok(())
        } else {
            Err(meta.error(
                "Allowed attributes are `pos` and `size` - Example: #[U8(pos=1, size=3)]"
                    .to_string(),
            ))
        }
    })
}

fn parse_with_attribute(attr: &syn::Attribute, size: &mut Option<usize>) -> Result<(), syn::Error> {
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("size") {
            let content;
            parenthesized!(content in meta.input);
            let lit: LitInt = content.parse()?;
            let n: usize = lit.base10_parse()?;
            *size = Some(n);
            Ok(())
        } else {
            let e =
                meta.error("Allowed attributes are `size` - Example: #[With(size(3))]".to_string());
            Err(e)
        }
    })
}

fn parse_from_field_attribute(
    attr: &syn::Attribute,
    field: &mut Option<proc_macro2::Ident>,
) -> Result<(), syn::Error> {
    attr.parse_nested_meta(|meta| {
        if let Some(name) = meta.path.get_ident().cloned() {
            *field = Some(name.to_owned());
            Ok(())
        } else {
            Err(meta.error(
                "Allowed attributes are `field_name` - Example: #[From(field_name)]".to_string(),
            ))
        }
    })
}

fn solve_for_inner_type(input: &syn::TypePath, identifier: &str) -> Option<syn::Type> {
    let syn::TypePath {
        path: syn::Path { segments, .. },
        ..
    } = input;
    let args = match &segments[0] {
        syn::PathSegment {
            ident,
            arguments:
                syn::PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }),
        } if ident == identifier && args.len() == 1 => args,
        _ => return None,
    };

    let inner_type = match &args[0] {
        syn::GenericArgument::Type(t) => t,
        _ => return None,
    };

    Some(inner_type.clone())
}

fn is_primitive_identity(ident: &syn::Ident) -> bool {
    PRIMITIVES.iter().any(|&primitive| ident == primitive)
}

fn is_primitive_type(tp: &syn::TypePath) -> bool {
    PRIMITIVES
        .iter()
        .any(|&primitive| tp.path.is_ident(primitive))
}

fn is_supported_primitive_type(tp: &syn::TypePath) -> bool {
    SUPPORTED_PRIMITIVES
        .iter()
        .any(|&primitive| tp.path.is_ident(primitive))
}

fn generate_chunks(n: usize, array_ident: proc_macro2::Ident) -> proc_macro2::TokenStream {
    let indices: Vec<_> = (0..n).map(|i| quote! { #array_ident[#i] }).collect();
    quote! { [ #( #indices ),* ] }
}
