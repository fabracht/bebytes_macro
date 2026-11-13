#![cfg_attr(not(feature = "std"), no_std)]

use proc_macro2::TokenStream;

pub fn validate_byte_completeness(fields: &syn::FieldsNamed) -> Result<(), TokenStream> {
    let mut total_bits = 0;
    let mut has_auto_sized = false;
    let mut bit_field_incomplete = false;

    for field in &fields.named {
        let field_has_bits = field.attrs.iter().any(|attr| attr.path().is_ident("bits"));

        if field_has_bits {
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
                            return Err(syn::Error::new_spanned(
                                attr,
                                "Invalid bits attribute format",
                            )
                            .to_compile_error());
                        }
                    }
                }
            }
            // Track if bit fields leave an incomplete byte
            bit_field_incomplete = total_bits % 8 != 0;
        } else if bit_field_incomplete {
            // If we have an incomplete byte from bit fields and now encounter a non-bit field,
            // that's an error because non-bit fields expect byte alignment
            return Err(syn::Error::new_spanned(
                field,
                "Bit fields must complete a full byte before any byte-aligned field. Add #[bits(N)] to this field or ensure previous bit fields sum to a multiple of 8 bits.",
            )
            .to_compile_error());
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
