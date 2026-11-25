#![cfg_attr(not(feature = "std"), no_std)]

use proc_macro2::TokenStream;

fn bits_attribute_size(attr: &syn::Attribute) -> Result<usize, TokenStream> {
    let syn::Meta::List(list) = &attr.meta else {
        return Err(
            syn::Error::new_spanned(attr, "Invalid bits attribute format").to_compile_error(),
        );
    };

    if list.tokens.is_empty() {
        return Err(syn::Error::new_spanned(
            attr,
            "Size required for bits attribute. Use #[bits(N)] where N is the number of bits needed for your field",
        )
        .to_compile_error());
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

    Ok(size)
}

pub fn validate_byte_completeness(fields: &syn::FieldsNamed) -> Result<(), TokenStream> {
    let mut total_bits = 0;

    for field in &fields.named {
        let mut field_has_bits = false;

        for attr in field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("bits"))
        {
            field_has_bits = true;
            total_bits += bits_attribute_size(attr)?;
        }

        if !field_has_bits && total_bits % 8 != 0 {
            return Err(syn::Error::new_spanned(
                field,
                "Bit fields must complete a full byte before any byte-aligned field. Ensure the preceding #[bits(N)] fields sum to a multiple of 8 bits, or give this field #[bits(N)] if it is an integer or char.",
            )
            .to_compile_error());
        }
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
