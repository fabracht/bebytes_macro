use quote::quote;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagType {
    U8,
    U16,
    U32,
    U64,
    U128,
}

impl FlagType {
    pub fn from_max_value(max: u128) -> Self {
        if max <= u128::from(u8::MAX) {
            FlagType::U8
        } else if max <= u128::from(u16::MAX) {
            FlagType::U16
        } else if max <= u128::from(u32::MAX) {
            FlagType::U32
        } else if max <= u128::from(u64::MAX) {
            FlagType::U64
        } else {
            FlagType::U128
        }
    }

    pub fn byte_size(self) -> usize {
        match self {
            FlagType::U8 => 1,
            FlagType::U16 => 2,
            FlagType::U32 => 4,
            FlagType::U64 => 8,
            FlagType::U128 => 16,
        }
    }

    pub fn type_tokens(self) -> proc_macro2::TokenStream {
        match self {
            FlagType::U8 => quote! { u8 },
            FlagType::U16 => quote! { u16 },
            FlagType::U32 => quote! { u32 },
            FlagType::U64 => quote! { u64 },
            FlagType::U128 => quote! { u128 },
        }
    }

    pub fn max_value(self) -> u128 {
        match self {
            FlagType::U8 => u128::from(u8::MAX),
            FlagType::U16 => u128::from(u16::MAX),
            FlagType::U32 => u128::from(u32::MAX),
            FlagType::U64 => u128::from(u64::MAX),
            FlagType::U128 => u128::MAX,
        }
    }

    pub fn type_name(self) -> &'static str {
        match self {
            FlagType::U8 => "u8",
            FlagType::U16 => "u16",
            FlagType::U32 => "u32",
            FlagType::U64 => "u64",
            FlagType::U128 => "u128",
        }
    }
}

// Type alias for the complex return type
type EnumHandleResult = (
    Vec<proc_macro2::TokenStream>,
    Vec<proc_macro2::TokenStream>,
    usize,
    Vec<proc_macro2::TokenStream>,
    Vec<(syn::Ident, u128)>,
    Vec<proc_macro2::TokenStream>, // errors
    FlagType,
);

#[allow(clippy::too_many_lines)]
pub fn handle_enum(
    mut errors: Vec<proc_macro2::TokenStream>,
    data_enum: syn::DataEnum,
    explicit_flag_type: Option<FlagType>,
) -> EnumHandleResult {
    let variants = data_enum.variants;
    let values = variants
        .iter()
        .enumerate()
        .map(|(index, variant)| {
            let ident = &variant.ident;

            match &variant.fields {
                syn::Fields::Named(_) => {
                    let error = syn::Error::new(
                        ident.span(),
                        "BeBytes does not support enums with struct variants. Use unit variants only.",
                    );
                    errors.push(error.to_compile_error());
                }
                syn::Fields::Unnamed(_) => {
                    let error = syn::Error::new(
                        ident.span(),
                        "BeBytes does not support enums with tuple variants. Use unit variants only.",
                    );
                    errors.push(error.to_compile_error());
                }
                syn::Fields::Unit => {}
            }

            let mut assigned_value = index as u128;

            if let Some((_, syn::Expr::Lit(expr_lit))) = &variant.discriminant {
                if let syn::Lit::Int(token) = &expr_lit.lit {
                    assigned_value = token.base10_parse::<u128>().unwrap_or_else(|_e| {
                        let error =
                            syn::Error::new(token.span(), "Failed to parse discriminant value");
                        errors.push(error.to_compile_error());
                        0
                    });
                }
            }
            (ident.clone(), assigned_value)
        })
        .collect::<Vec<_>>();

    let max_discriminant = values.iter().map(|(_, value)| *value).max().unwrap_or(0);
    let detected_flag_type = FlagType::from_max_value(max_discriminant);
    let flag_type = explicit_flag_type.unwrap_or(detected_flag_type);
    let byte_size = flag_type.byte_size();

    #[allow(clippy::cast_possible_truncation)]
    let typed_values: Vec<_> = values
        .iter()
        .map(|(ident, val)| {
            let typed_val = match flag_type {
                FlagType::U8 => {
                    let v = *val as u8;
                    quote! { #v }
                }
                FlagType::U16 => {
                    let v = *val as u16;
                    quote! { #v }
                }
                FlagType::U32 => {
                    let v = *val as u32;
                    quote! { #v }
                }
                FlagType::U64 => {
                    let v = *val as u64;
                    quote! { #v }
                }
                FlagType::U128 => {
                    let v = *val;
                    quote! { #v }
                }
            };
            (ident.clone(), typed_val, *val)
        })
        .collect();

    let from_bytes_arms = typed_values
        .iter()
        .map(|(ident, typed_val, _)| {
            quote! {
                #typed_val => Ok((Self::#ident, #byte_size)),
            }
        })
        .collect::<Vec<_>>();

    let to_bytes_arms = typed_values
        .iter()
        .map(|(ident, typed_val, _)| {
            quote! {
                Self::#ident => #typed_val,
            }
        })
        .collect::<Vec<_>>();

    let min_bits = if max_discriminant == 0 {
        1
    } else {
        let mut bits = 0;
        let mut val = max_discriminant;
        while val > 0 {
            bits += 1;
            val >>= 1;
        }
        bits
    };

    let try_from_arms = typed_values
        .iter()
        .map(|(ident, typed_val, _)| {
            quote! {
                #typed_val => Ok(Self::#ident),
            }
        })
        .collect::<Vec<_>>();

    (
        from_bytes_arms,
        to_bytes_arms,
        min_bits,
        try_from_arms,
        values,
        errors,
        flag_type,
    )
}
