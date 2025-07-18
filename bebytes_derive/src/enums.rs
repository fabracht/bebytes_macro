use quote::quote;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

// Type alias for the complex return type
type EnumHandleResult = (
    Vec<proc_macro2::TokenStream>,
    Vec<proc_macro2::TokenStream>,
    usize,
    Vec<proc_macro2::TokenStream>,
    Vec<(syn::Ident, u8)>,
    Vec<proc_macro2::TokenStream>, // errors
);

pub fn handle_enum(
    mut errors: Vec<proc_macro2::TokenStream>,
    data_enum: syn::DataEnum,
) -> EnumHandleResult {
    let variants = data_enum.variants;
    let values = variants
        .iter()
        .enumerate()
        .map(|(index, variant)| {
            let ident = &variant.ident;
            let mut assigned_value = u8::try_from(index).unwrap_or_else(|_| {
                let error = syn::Error::new(
                    ident.span(),
                    format!("Enum variant index {index} exceeds u8 range"),
                );
                errors.push(error.to_compile_error());
                0
            });
            if let Some((_, syn::Expr::Lit(expr_lit))) = &variant.discriminant {
                if let syn::Lit::Int(token) = &expr_lit.lit {
                    // First parse as usize to check the actual value
                    let value: usize = token.base10_parse().unwrap_or_else(|_e| {
                        let error =
                            syn::Error::new(token.span(), "Failed to parse discriminant value");
                        errors.push(error.to_compile_error());
                        0
                    });

                    // Check if value exceeds u8 range
                    if value > 255 {
                        let error = syn::Error::new(
                            token.span(),
                            format!(
                                "Enum discriminant value {value} exceeds u8 range (0-255). \
                                Consider using #[repr(u8)] to make this constraint explicit, \
                                or ensure all discriminants fit within u8 range."
                            ),
                        );
                        errors.push(error.to_compile_error());
                        assigned_value = 0; // Use 0 as fallback to continue compilation
                    } else {
                        assigned_value = u8::try_from(value).unwrap_or(0);
                    }
                }
            }
            (ident.clone(), assigned_value)
        })
        .collect::<Vec<_>>();

    let from_bytes_arms = values
        .iter()
        .map(|(ident, assigned_value)| {
            quote! {
                #assigned_value => Ok((Self::#ident, 1)),
            }
        })
        .collect::<Vec<_>>();

    let to_bytes_arms = values
        .iter()
        .map(|(ident, assigned_value)| {
            quote! {
                Self::#ident => #assigned_value as u8,
            }
        })
        .collect::<Vec<_>>();

    // Calculate minimum bits needed for this enum
    let max_discriminant = values.iter().map(|(_, value)| *value).max().unwrap_or(0);

    // Calculate minimum bits: ceil(log2(max_discriminant + 1))
    let min_bits = if max_discriminant == 0 {
        1 // Even a single variant needs at least 1 bit
    } else {
        // Find the position of the highest set bit
        let mut bits = 0;
        let mut val = max_discriminant;
        while val > 0 {
            bits += 1;
            val >>= 1;
        }
        bits
    };

    // Generate TryFrom<u8> arms for auto-sized bit fields
    let try_from_arms = values
        .iter()
        .map(|(ident, assigned_value)| {
            quote! {
                #assigned_value => Ok(Self::#ident),
            }
        })
        .collect::<Vec<_>>();

    // For enums, the byte representation is the same for both endianness since we're just storing a single byte value
    (
        from_bytes_arms,
        to_bytes_arms,
        min_bits,
        try_from_arms,
        values,
        errors,
    )
}
