#![cfg_attr(not(feature = "std"), no_std)]

use proc_macro2::TokenStream;

fn bits_attribute_size(attr: &syn::Attribute) -> Result<Option<usize>, TokenStream> {
    let syn::Meta::List(list) = &attr.meta else {
        return Err(
            syn::Error::new_spanned(attr, "Invalid bits attribute format").to_compile_error(),
        );
    };

    if list.tokens.is_empty() {
        return Ok(None);
    }

    let literal = attr
        .parse_args::<syn::LitInt>()
        .map_err(|e| e.to_compile_error())?;
    let size = literal
        .base10_parse::<usize>()
        .map_err(|e| e.to_compile_error())?;

    if size == 0 {
        return Err(
            syn::Error::new_spanned(attr, "bits attribute must specify at least 1 bit")
                .to_compile_error(),
        );
    }

    Ok(Some(size))
}

pub fn validate_byte_completeness(fields: &syn::FieldsNamed) -> Result<(), TokenStream> {
    let mut total_bits = 0;
    let mut has_auto_sized = false;

    for field in &fields.named {
        let mut field_has_bits = false;

        for attr in field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("bits"))
        {
            field_has_bits = true;
            match bits_attribute_size(attr)? {
                Some(size) => total_bits += size,
                None => has_auto_sized = true,
            }
        }

        if !field_has_bits && !has_auto_sized && total_bits % 8 != 0 {
            return Err(syn::Error::new_spanned(
                field,
                "Bit fields must complete a full byte before any byte-aligned field. Ensure the preceding #[bits(N)] fields sum to a multiple of 8 bits, or give this field #[bits(N)] if it is an integer or char.",
            )
            .to_compile_error());
        }
    }

    if has_auto_sized {
        return Ok(());
    }

    if total_bits % 8 != 0 {
        return Err(syn::Error::new_spanned(
            fields,
            format!(
                "bits attributes must complete a full byte. Total bits: {total_bits}, which is {} bits short of a complete byte",
                8 - (total_bits % 8)
            ),
        )
        .to_compile_error());
    }

    Ok(())
}
