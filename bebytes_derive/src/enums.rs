use quote::quote;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub fn handle_enum(
    mut errors: Vec<proc_macro2::TokenStream>,
    data_enum: syn::DataEnum,
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
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
                        let error = syn::Error::new(token.span(), "Failed to parse token value");
                        errors.push(error.to_compile_error());
                        0
                    });
                }
            };
            (ident, assigned_value)
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

    // For enums, the byte representation is the same for both endianness since we're just storing a single byte value
    (from_bytes_arms, to_bytes_arms)
}
