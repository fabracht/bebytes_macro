use quote::quote;
use syn::{FieldsNamed, Type};

/// Generate raw pointer-based direct writing for maximum performance
/// This approach eliminates all abstraction overhead by writing directly to memory
pub fn generate_raw_pointer_writing(
    field_name: &syn::Ident,
    field_type: &Type,
    endianness: crate::consts::Endianness,
    offset_var: &syn::Ident,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let field_size = crate::utils::get_primitive_type_size(field_type)?;

    // Handle char type specially
    if let Type::Path(tp) = field_type {
        if tp.path.is_ident("char") {
            return match endianness {
                crate::consts::Endianness::Big => Ok(quote! {
                    let char_bytes = (self.#field_name as u32).to_be_bytes();
                    ::core::ptr::copy_nonoverlapping(char_bytes.as_ptr(), ptr.add(#offset_var), 4);
                    #offset_var += 4;
                }),
                crate::consts::Endianness::Little => Ok(quote! {
                    let char_bytes = (self.#field_name as u32).to_le_bytes();
                    ::core::ptr::copy_nonoverlapping(char_bytes.as_ptr(), ptr.add(#offset_var), 4);
                    #offset_var += 4;
                }),
            };
        }
    }

    match field_size {
        1 => Ok(quote! {
            *ptr.add(#offset_var) = self.#field_name as u8;
            #offset_var += 1;
        }),
        2 => match endianness {
            crate::consts::Endianness::Big => Ok(quote! {
                let field_bytes = (self.#field_name as u16).to_be_bytes();
                ::core::ptr::copy_nonoverlapping(field_bytes.as_ptr(), ptr.add(#offset_var), 2);
                #offset_var += 2;
            }),
            crate::consts::Endianness::Little => Ok(quote! {
                let field_bytes = (self.#field_name as u16).to_le_bytes();
                ::core::ptr::copy_nonoverlapping(field_bytes.as_ptr(), ptr.add(#offset_var), 2);
                #offset_var += 2;
            }),
        },
        4 => match endianness {
            crate::consts::Endianness::Big => Ok(quote! {
                let field_bytes = (self.#field_name as u32).to_be_bytes();
                ::core::ptr::copy_nonoverlapping(field_bytes.as_ptr(), ptr.add(#offset_var), 4);
                #offset_var += 4;
            }),
            crate::consts::Endianness::Little => Ok(quote! {
                let field_bytes = (self.#field_name as u32).to_le_bytes();
                ::core::ptr::copy_nonoverlapping(field_bytes.as_ptr(), ptr.add(#offset_var), 4);
                #offset_var += 4;
            }),
        },
        8 => match endianness {
            crate::consts::Endianness::Big => Ok(quote! {
                let field_bytes = (self.#field_name as u64).to_be_bytes();
                ::core::ptr::copy_nonoverlapping(field_bytes.as_ptr(), ptr.add(#offset_var), 8);
                #offset_var += 8;
            }),
            crate::consts::Endianness::Little => Ok(quote! {
                let field_bytes = (self.#field_name as u64).to_le_bytes();
                ::core::ptr::copy_nonoverlapping(field_bytes.as_ptr(), ptr.add(#offset_var), 8);
                #offset_var += 8;
            }),
        },
        16 => match endianness {
            crate::consts::Endianness::Big => Ok(quote! {
                let field_bytes = (self.#field_name as u128).to_be_bytes();
                ::core::ptr::copy_nonoverlapping(field_bytes.as_ptr(), ptr.add(#offset_var), 16);
                #offset_var += 16;
            }),
            crate::consts::Endianness::Little => Ok(quote! {
                let field_bytes = (self.#field_name as u128).to_le_bytes();
                ::core::ptr::copy_nonoverlapping(field_bytes.as_ptr(), ptr.add(#offset_var), 16);
                #offset_var += 16;
            }),
        },
        _ => Err(syn::Error::new_spanned(
            field_type,
            "Unsupported primitive type size for raw pointer writing",
        )),
    }
}

/// Generate raw pointer writing for byte arrays
pub fn generate_raw_pointer_array_writing(
    field_name: &syn::Ident,
    array_length: usize,
    offset_var: &syn::Ident,
) -> proc_macro2::TokenStream {
    quote! {
        ::core::ptr::copy_nonoverlapping(self.#field_name.as_ptr(), ptr.add(#offset_var), #array_length);
        #offset_var += #array_length;
    }
}

/// Generate raw pointer writing for an entire struct
pub fn generate_raw_pointer_struct_writing(
    fields: &FieldsNamed,
    endianness: crate::consts::Endianness,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let mut field_writing_code = Vec::new();
    let offset_var = syn::Ident::new("offset", proc_macro2::Span::call_site());

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        // Check if this is a bit field (has #[bits(N)] attribute)
        let is_bit_field = field.attrs.iter().any(|attr| attr.path().is_ident("bits"));

        if is_bit_field {
            // Skip bit fields for now to keep implementation simple
            continue;
        }

        // Handle primitive types and arrays
        if let Ok(writing_code) =
            generate_raw_pointer_writing(field_name, field_type, endianness, &offset_var)
        {
            field_writing_code.push(writing_code);
        } else if let Type::Array(array_type) = field_type {
            // Handle byte arrays
            if let Type::Path(element_type) = &*array_type.elem {
                if element_type.path.is_ident("u8") {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(len),
                        ..
                    }) = &array_type.len
                    {
                        let array_len: usize = len.base10_parse()?;
                        let array_code =
                            generate_raw_pointer_array_writing(field_name, array_len, &offset_var);
                        field_writing_code.push(array_code);
                    }
                }
            }
        }
    }

    Ok(quote! {
        #(#field_writing_code)*
    })
}
