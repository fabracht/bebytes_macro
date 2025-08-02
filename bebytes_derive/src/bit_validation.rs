#![cfg_attr(not(feature = "std"), no_std)]

use proc_macro2::TokenStream;

pub fn validate_byte_completeness(fields: &syn::FieldsNamed) -> Result<(), TokenStream> {
    let mut total_bits = 0;
    let mut has_auto_sized = false;

    for field in &fields.named {
        for attr in &field.attrs {
            if attr.path().is_ident("bits") {
                // Parse #[bits(N)] where N is the size
                // For validation, we need to know the actual size
                // Auto-sized fields (#[bits()]) will be handled later
                match &attr.meta {
                    syn::Meta::List(list) => {
                        if list.tokens.is_empty() {
                            // #[bits()] - auto-sized, skip validation for now
                            // The actual validation will happen during code generation
                            has_auto_sized = true;
                            continue;
                        }

                        // Try to parse as integer
                        match attr.parse_args::<syn::LitInt>() {
                            Ok(literal) => match literal.base10_parse::<usize>() {
                                Ok(n) => {
                                    if n == 0 {
                                        return Err(syn::Error::new_spanned(
                                            attr,
                                            "bits attribute must specify at least 1 bit",
                                        )
                                        .to_compile_error());
                                    }
                                    total_bits += n;
                                }
                                Err(e) => return Err(e.to_compile_error()),
                            },
                            Err(e) => return Err(e.to_compile_error()),
                        }
                    }
                    _ => {
                        // This shouldn't happen due to Rust's validation
                        return Err(
                            syn::Error::new_spanned(attr, "Invalid bits attribute format")
                                .to_compile_error(),
                        );
                    }
                }
            }
        }
    }

    // If there are auto-sized fields, we can't validate at compile time
    // The validation will happen during code generation
    if has_auto_sized {
        return Ok(());
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
