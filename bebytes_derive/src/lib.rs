extern crate alloc;
extern crate proc_macro;
use alloc::vec::Vec;
use proc_macro::TokenStream;
use quote::{__private::Span, quote, quote_spanned};
use syn::{
    parenthesized, parse_macro_input, spanned::Spanned, AngleBracketedGenericArguments, Data,
    DeriveInput, Fields, LitInt,
};

const PRIMITIVES: [&str; 17] = [
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
    "f64", "bool", "char", "str",
];
const SUPPORTED_PRIMITIVES: [&str; 10] = [
    "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128",
];

// BeBytes makes your bit shifting life a thing of the past
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
    // let mut lifetime_params = Vec::new();
    match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                let struct_field_names = fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();

                // Total size tracks the total size of the U8 attributes
                let mut total_size: usize = 0;
                // Check for the last field. This is used to check if the Vec is the last field
                // We can't have a Vec in the middle of a struct because we have no way of dealing
                // with the variable size of the Vec
                let last_field = fields.named.last();
                let mut is_last_field = false;

                for field in fields.named.clone().into_iter() {
                    if let Some(last_field) = last_field {
                        is_last_field = last_field.ident == field.ident;
                    }
                    // initialize u8 flag to false
                    let mut u8_attribute_present = false;

                    // get the attributes of the field
                    let attributes = field.attrs.clone();

                    // get the name of the field
                    let field_name = field.ident.clone().unwrap();
                    // get the type of the field
                    let field_type = &field.ty;
                    let (pos, mut size, vec_size_ident) =
                        parse_attributes(attributes, &mut u8_attribute_present, &mut errors);

                    // check if the U8 attribute is present
                    if u8_attribute_present {
                        match field_type {
                            // if field is U8, we apply bit shifting
                            syn::Type::Path(tp) if !is_supported_primitive_type(tp) => {
                                let error = syn::Error::new(
                                    field_type.span(),
                                    "Unsupported type for U8 attribute",
                                );
                                errors.push(error.to_compile_error());
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
                            }); // retrieve position and size from attributes
                        if pos.is_none() && size.is_none() {
                            let error = syn::Error::new(
                                field.span(),
                                "U8 attribute must have a size and a position",
                            );
                            errors.push(error.to_compile_error());
                        }
                        // Deal with the position and size
                        if let (Some(pos), Some(size)) = (pos, size) {
                            bit_sum.push(quote! {bit_sum += #size;});

                            // set the bit mask
                            let mask: u128 = (1 << size) - 1;
                            // add runtime check if the value requested is in the valid range for that type
                            field_limit_check.push(quote! {
                                if #field_name > #mask as #field_type {
                                    let err_msg = format!(
                                        "Value of field {} is out of range (max value: {})",
                                        stringify!(#field_name),
                                        #mask
                                    );
                                    let err = std::io::Error::new(std::io::ErrorKind::Other, err_msg);
                                    panic!("{}", err);
                                }
                            });

                            // check if the position is in sequence
                            if pos % 8 != total_size % 8 {
                                let message = format!(
                                "U8 attributes must obey the sequence specified by the previous attributes. Expected position {} but got {}",
                                total_size, pos
                            );
                                errors.push(
                                    syn::Error::new_spanned(&field, message).to_compile_error(),
                                );
                            }
                            // add the parsing code for the field
                            if number_length > 1 {
                                let chunks = generate_chunks(
                                    number_length,
                                    syn::Ident::new("chunk", Span::call_site()),
                                );

                                field_parsing.push(quote! {
                                    let mut inner_total_size = #total_size;
                                    // Initialize the field
                                    let mut #field_name = 0 as #field_type;
                                    // In order to use the mask, we need to reset the multi-byte
                                    // field to it's original position
                                    // To do that, we can iterate over chunks of the bytes array
                                    bytes.chunks(#number_length).for_each(|chunk| {
                                        // First we parse the chunk into the field type
                                        let u_type = #field_type::from_be_bytes(#chunks);
                                        // Then we shift the u_type to the right based on its actual size
                                        // If the field size attribute is 14, we need to shift 2 bytes to the right
                                        // If the field size attribute is 16, we need to shift 0 bytes to the right
                                        let shift_left = _bit_sum % 8;
                                        let left_shifted_u_type = u_type << shift_left;
                                        let shift_right = 8 * #number_length - #size;
                                        let shifted_u_type = left_shifted_u_type >> shift_right;
                                        // Then we mask the shifted value to delete unwanted bits
                                        // and that becomes the field value
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
                                    // println!("Field name {}", stringify!(#field_name));
                                    let masked_value = #field_name & #mask as #field_type;
                                    // The shift factor tells us about the current position in the byte
                                    // It's the size of the number in bits minus the size requested in bits
                                    // plus the current position in the byte
                                    let shift_left = (#number_length * 8) - #size;
                                    // println!("shift_left: {}", shift_left);
                                    let shift_right = (#pos % 8);
                                    // println!("shift_right: {}", shift_right);
                                    // The shifted value aligns the value with the current position in the byte
                                    let shifted_masked_value = (masked_value << shift_left) >> shift_right;
                                    // println!("shifted_masked_value: {:08b}", shifted_masked_value);
                                    // We split the value into bytes
                                    let byte_values = #field_type::to_be_bytes(shifted_masked_value);
                                    // println!("byte_values: {:?}", byte_values);
                                    // Iterating over the bytes. The first byte always fills a byte completely.
                                    // The following bytes will fill the second, third, ... byte and so on. So,
                                    // we need to increase the index value in the bytes array by the index of the
                                    // current byte in the input sequence.
                                    // The last byte may or may not fill the byte completely.
                                    for i in 0..#number_length {
                                        if bytes.len() <= _bit_sum / 8 + i {
                                            bytes.resize(_bit_sum / 8 + i + 1, 0);
                                        }
                                        bytes[_bit_sum / 8 + i] |= byte_values[i];
                                        // println!("bytes[{}]: {:08b}", _bit_sum / 8 + i, bytes[_bit_sum / 8 + i]);
                                    }
                                    _bit_sum += #size;
                                });
                            } else {
                                field_parsing.push(quote! {
                                    let #field_name = (bytes[_bit_sum / 8] as #field_type >> (7 - (#size + #pos % 8 - 1) as #field_type )) & (#mask as #field_type);
                                    _bit_sum += #size;
                                });
                                // add the writing code for the field
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
                                    // println!("field_name {} left shift {} bytes[{}]", stringify!(#field_name), (7 - (#size - 1) - #pos % 8 ), _bit_sum / 8);
                                    _bit_sum += #size;
                                });
                            }
                            total_size += size;
                        }
                    } else {
                        // if field is not U8, total_size has to be a multiple of 8
                        if total_size % 8 != 0 {
                            errors.push(
                                syn::Error::new_spanned(
                                    &field,
                                    "U8 attributes must add up to 8 before any other non U8 field is added",
                                )
                                .to_compile_error(),
                            );
                        }
                        // supported primitive types
                        match field_type {
                            // if field is number type, we apply be bytes conversion
                            syn::Type::Path(tp) if is_primitive_type(tp) =>
                            {
                                named_fields.push(quote! { let #field_name = self.#field_name; });

                                // get the size of the number in bytes
                                let field_size =
                                    match get_number_size(field_type, &field, &mut errors) {
                                        Some(value) => value,
                                        None => continue,
                                    };
                                bit_sum.push(quote! { bit_sum += #field_size * 8;});
                                // write the parse and writing code for the field
                                parse_write_number(
                                    field_size,
                                    &mut field_parsing,
                                    &field_name,
                                    field_type,
                                    &mut field_writing,
                                );

                            }
                            syn::Type::Array(tp) => { // if field is an Array
                                named_fields.push(quote! { let #field_name = self.#field_name.to_owned(); });

                                // get the size of the array
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
                                    // Retrieve type segments
                                    let syn::TypePath {
                                        path: syn::Path { segments, .. },
                                        ..
                                    } = elem;
                                    // Array must be of bytes
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
                            // if field is a non-empty Vec and is made of a primitive type
                            syn::Type::Path(tp)
                                if !tp.path.segments.is_empty()
                                    && tp.path.segments[0].ident == "Vec" =>
                            {
                                named_fields.push(quote! { let #field_name = self.#field_name.to_owned(); });

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
                                    if is_primitive_type(inner_tp)
                                    {
                                        if let Some(vec_size_ident) = vec_size_ident {
                                            bit_sum.push(quote! {
                                                bit_sum = 2<<16 as usize;
                                            });

                                            field_parsing.push(quote! {
                                                let vec_length = #vec_size_ident as usize;
                                                byte_index = _bit_sum / 8;
                                                let end_index = byte_index + vec_length;
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
                                                bit_sum = 2<<16 as usize;
                                            });
                                            field_parsing.push(quote! {
                                                byte_index = _bit_sum / 8;
                                                let #field_name = Vec::from(&bytes[byte_index..]);
                                                _bit_sum += #field_name.len() * 8;
                                            });
                                            // If the current field is not the last field, raise an error
                                            if !is_last_field {
                                                let error = syn::Error::new(
                                                    field.ty.span(),
                                                    "Vectors can only be used for padding the end of a struct",
                                                );
                                                errors.push(error.to_compile_error());
                                            }
                                        }
                                        field_writing.push(quote! {
                                            let field_length = &#field_name.len();
                                            // Vec type
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
                            syn::Type::Path(tp) // if field is a non-empty Option
                                if !tp.path.segments.is_empty()
                                    && tp.path.segments[0].ident == "Option" =>
                            {
                                if !tp.path.segments.is_empty()
                                    && tp.path.segments[0].ident == "Option"
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
                                        if is_primitive_type(inner_tp)
                                        {
                                            // get the size of the number in bytes
                                            let field_size = match get_number_size(
                                                &inner_type,
                                                &field,
                                                &mut errors,
                                            ) {
                                                Some(value) => value,
                                                None => continue,
                                            };
                                            bit_sum.push(quote! { bit_sum += 8 * #field_size;});
                                            field_parsing.push(quote! {
                                                // Option type
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
                            }
                            syn::Type::Path(tp)  // Struct case
                                if !tp.path.segments.is_empty()
                                    && !is_primitive_identity(&tp.path.segments[0].ident) =>
                            {
                                named_fields.push(quote! { let #field_name = self.#field_name.to_owned(); });

                                bit_sum.push(quote! { bit_sum += 8 * #field_type::field_size();});
                                field_parsing.push(quote_spanned! { field.span() =>
                                    byte_index = _bit_sum / 8;
                                    // let predicted_size = core::mem::size_of::<#field_type>();
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
                                let error_message =
                                    format!("Unsupported type for field {}", field_name);
                                let error = syn::Error::new(field.ty.span(), error_message);
                                errors.push(error.to_compile_error());
                            }
                        }
                    }
                }

                // Generate the code for the constructor
                let constructor_arg_list = fields.named.iter().map(|f| {
                    let field_ident = &f.ident;
                    let field_type = &f.ty;
                    quote! { #field_ident: #field_type }
                });
                let expanded = quote! {
                    impl #my_trait_path for #name {
                        fn try_from_be_bytes(bytes: &[u8]) -> Result<(Self, usize), Box<dyn std::error::Error>> {
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

                    impl  #name {
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

            let from_be_bytes_arms = values.iter().map(|(ident, assigned_value)| {
                quote! {
                    #assigned_value => Ok((Self::#ident, 1)),
                }
            });

            let to_be_bytes_arms = values.iter().map(|(ident, assigned_value)| {
                quote! {
                    Self::#ident => #assigned_value as u8,
                }
            });
            // Generate the code for the enum
            let expanded = quote! {
                impl #my_trait_path for #name {
                    fn try_from_be_bytes(bytes: &[u8]) -> Result<(Self, usize), Box<dyn std::error::Error>> {
                        if bytes.is_empty() {
                            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "No bytes provided.")));
                        }

                        let value = bytes[0];
                        match value {
                            #(#from_be_bytes_arms)*
                            _ => Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("No matching variant found for value {}", value)))),
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
                        std::mem::size_of::<Self>()
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
    field_parsing: &mut Vec<quote::__private::TokenStream>,
    field_name: &syn::Ident,
    field_type: &syn::Type,
    field_writing: &mut Vec<quote::__private::TokenStream>,
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
        // bytes[#byte_index..#end_byte_index].copy_from_slice(&#field_name.to_be_bytes());
        let field_slice = &#field_name.to_be_bytes();
        bytes.extend_from_slice(field_slice);
        _bit_sum += field_slice.len() * 8;
    });
}

fn get_number_size(
    field_type: &syn::Type,
    field: &syn::Field,
    errors: &mut Vec<quote::__private::TokenStream>,
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
    errors: &mut Vec<quote::__private::TokenStream>,
) -> (Option<usize>, Option<usize>, Option<proc_macro2::Ident>) {
    let mut pos = None;
    let mut size = None;
    let mut field = None;
    for attr in attributes {
        if attr.path().is_ident("U8") {
            *u8_attribute_present = true;
            let nested_result = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("pos") || meta.path.is_ident("size") {
                    if meta.path.is_ident("pos") {
                        let content;
                        parenthesized!(content in meta.input);
                        let lit: LitInt = content.parse()?;
                        let n: usize = lit.base10_parse()?;
                        pos = Some(n);
                        return Ok(());
                    }
                    if meta.path.is_ident("size") {
                        let content;
                        parenthesized!(content in meta.input);
                        let lit: LitInt = content.parse()?;
                        let n: usize = lit.base10_parse()?;
                        size = Some(n);
                        return Ok(());
                    }
                } else {
                    return Err(meta.error(
                        "Allowed attributes are `pos` and `size` - Example: #[U8(pos=1, size=3)]"
                            .to_string(),
                    ));
                }
                Ok(())
            });
            if let Err(e) = nested_result {
                errors.push(e.to_compile_error());
            }
        } else if attr.path().is_ident("With") {
            if let Err(e) = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("size") {
                    let content;
                    parenthesized!(content in meta.input);
                    let lit: LitInt = content.parse()?;
                    let n: usize = lit.base10_parse()?;
                    size = Some(n);
                } else {
                    let e = meta.error(
                        "Allowed attributes are `size` - Example: #[With(size(3))]".to_string(),
                    );
                    errors.push(e.to_compile_error());
                    return Err(e);
                }
                Ok(())
            }) {
                errors.push(e.to_compile_error());
            };
        } else if attr.path().is_ident("FromField") {
            let _ = attr.parse_nested_meta(|meta| {
                if let Some(name) = meta.path.get_ident().cloned() {
                    field = Some(name.to_owned());
                } else {
                    return Err(meta.error(
                        "Allowed attributes are `field_name` - Example: #[From(field_name)]"
                            .to_string(),
                    ));
                }
                Ok(())
            });
        }
    }
    (pos, size, field)
}

/// Given a type and an identifier, `solve_for_inner_type` attempts to retrieve the inner type of the input type
/// that is wrapped by the provided identifier. If the input type does not contain the specified identifier or
/// has more than one generic argument, the function returns `None`.
fn solve_for_inner_type(input: &syn::TypePath, identifier: &str) -> Option<syn::Type> {
    // Retrieve type segments
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

// Helper function to check if a given identifier is a primitive type
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
