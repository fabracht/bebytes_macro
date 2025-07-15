#![cfg_attr(not(feature = "std"), no_std)]

use proc_macro2::TokenStream;

pub fn validate_byte_completeness(fields: &syn::FieldsNamed) -> Result<(), TokenStream> {
    let mut total_bits = 0;

    for field in &fields.named {
        for attr in &field.attrs {
            if attr.path().is_ident("bits") {
                // Parse #[bits(N)] where N is the size
                match attr.parse_args::<syn::LitInt>() {
                    Ok(lit) => match lit.base10_parse::<usize>() {
                        Ok(n) => total_bits += n,
                        Err(e) => return Err(e.to_compile_error()),
                    },
                    Err(e) => return Err(e.to_compile_error()),
                }
            }
        }
    }

    // Check if bits complete a full byte
    if total_bits % 8 != 0 {
        return Err(syn::Error::new_spanned(
            fields,
            format!(
                "bits attributes must complete a full byte. Total bits: {}, which is {} bits short of a complete byte",
                total_bits,
                8 - (total_bits % 8)
            ),
        )
        .to_compile_error());
    }

    Ok(())
}
