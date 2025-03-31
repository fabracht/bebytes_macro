use syn::{parenthesized, LitInt};

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub fn parse_attributes(
    attributes: Vec<syn::Attribute>,
    u8_attribute_present: &mut bool,
    errors: &mut Vec<proc_macro2::TokenStream>,
) -> (Option<usize>, Option<usize>, Option<proc_macro2::Ident>) {
    let mut pos = None;
    let mut size = None;
    let mut field = None;

    for attr in attributes {
        if attr.path().is_ident("U8") {
            *u8_attribute_present = true;
            if let Err(e) = parse_u8_attribute(&attr, &mut pos, &mut size) {
                errors.push(e.to_compile_error());
            }
        } else if attr.path().is_ident("With") {
            if let Err(e) = parse_with_attribute(&attr, &mut size) {
                errors.push(e.to_compile_error());
            }
        } else if attr.path().is_ident("FromField") {
            if let Err(e) = parse_from_field_attribute(&attr, &mut field) {
                errors.push(e.to_compile_error());
            }
        }
    }

    (pos, size, field)
}

pub fn parse_u8_attribute(
    attr: &syn::Attribute,
    pos: &mut Option<usize>,
    size: &mut Option<usize>,
) -> Result<(), syn::Error> {
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("pos") {
            let content;
            parenthesized!(content in meta.input);
            let lit: LitInt = content.parse()?;
            let n: usize = lit.base10_parse()?;
            *pos = Some(n);
            Ok(())
        } else if meta.path.is_ident("size") {
            let content;
            parenthesized!(content in meta.input);
            let lit: LitInt = content.parse()?;
            let n: usize = lit.base10_parse()?;
            *size = Some(n);
            Ok(())
        } else {
            Err(meta
                .error("Allowed attributes are `pos` and `size` - Example: #[U8(pos=1, size=3)]"))
        }
    })
}

pub fn parse_with_attribute(
    attr: &syn::Attribute,
    size: &mut Option<usize>,
) -> Result<(), syn::Error> {
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("size") {
            let content;
            parenthesized!(content in meta.input);
            let lit: LitInt = content.parse()?;
            let n: usize = lit.base10_parse()?;
            *size = Some(n);
            Ok(())
        } else {
            let e = meta.error("Allowed attributes are `size` - Example: #[With(size(3))]");
            Err(e)
        }
    })
}

pub fn parse_from_field_attribute(
    attr: &syn::Attribute,
    field: &mut Option<proc_macro2::Ident>,
) -> Result<(), syn::Error> {
    attr.parse_nested_meta(|meta| {
        if let Some(name) = meta.path.get_ident().cloned() {
            *field = Some(name.to_owned());
            Ok(())
        } else {
            Err(meta.error("Allowed attributes are `field_name` - Example: #[From(field_name)]"))
        }
    })
}
