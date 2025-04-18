use syn::{parenthesized, spanned::Spanned, LitInt};

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Represents all possible field attributes for `BeBytes`
#[derive(Default, Clone)]
pub struct FieldAttributes {
    // U8 attributes for bit-level fields
    pub u8_pos: Option<usize>,
    pub u8_size: Option<usize>,

    // With(size()) attribute for fixed-size vectors
    pub with_size: Option<usize>,

    // FromField attribute for linking vector size to another field
    pub from_field: Option<proc_macro2::Ident>,
}

impl FieldAttributes {
    pub fn has_u8_attributes(&self) -> bool {
        self.u8_pos.is_some() || self.u8_size.is_some()
    }

    pub fn is_bit_field(&self) -> bool {
        // A valid bit field needs both position and size
        self.u8_pos.is_some() && self.u8_size.is_some()
    }
}

/// Extract all field attributes from a list of attributes
pub fn extract_field_attributes(
    attributes: &[syn::Attribute],
    errors: &mut Vec<proc_macro2::TokenStream>,
) -> FieldAttributes {
    let mut field_attrs = FieldAttributes::default();

    for attr in attributes {
        if attr.path().is_ident("U8") {
            if let Err(e) =
                parse_u8_attribute(attr, &mut field_attrs.u8_pos, &mut field_attrs.u8_size)
            {
                errors.push(e.to_compile_error());
            }
        } else if attr.path().is_ident("With") {
            if let Err(e) = parse_with_attribute(attr, &mut field_attrs.with_size) {
                errors.push(e.to_compile_error());
            }
        } else if attr.path().is_ident("FromField") {
            if let Err(e) = parse_from_field_attribute(attr, &mut field_attrs.from_field) {
                errors.push(e.to_compile_error());
            }
        }
    }

    field_attrs
}

/// Map of field names to their attributes
pub type FieldAttributesMap = std::collections::HashMap<String, FieldAttributes>;

/// Extract all field attributes for all fields in a struct
pub fn extract_struct_field_attributes(
    fields_named: &syn::FieldsNamed,
    errors: &mut Vec<proc_macro2::TokenStream>,
) -> FieldAttributesMap {
    let mut field_attrs_map = std::collections::HashMap::new();

    for field in &fields_named.named {
        if let Some(field_ident) = &field.ident {
            let field_name = field_ident.to_string();
            let attrs = extract_field_attributes(&field.attrs, errors);
            field_attrs_map.insert(field_name, attrs);
        }
    }

    field_attrs_map
}

/// Validate that `FromField` references exist in the struct
pub fn validate_from_field_references(
    fields_named: &syn::FieldsNamed,
    field_attrs_map: &FieldAttributesMap,
    errors: &mut Vec<proc_macro2::TokenStream>,
) -> bool {
    let mut has_errors = false;

    for field in &fields_named.named {
        if let Some(field_ident) = &field.ident {
            let field_name = field_ident.to_string();

            if let Some(attrs) = field_attrs_map.get(&field_name) {
                if let Some(from_field) = &attrs.from_field {
                    let from_field_name = from_field.to_string();
                    if !field_attrs_map.contains_key(&from_field_name) {
                        let error = syn::Error::new(
                            field.span(),
                            format!("FromField references non-existent field '{from_field_name}'"),
                        );
                        errors.push(error.to_compile_error());
                        has_errors = true;
                    }
                }
            }
        }
    }

    !has_errors
}

/// Parse U8 attribute for bit-level field definitions
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
                .error("Allowed attributes are `pos` and `size` - Example: #[U8(pos(1), size(3))]"))
        }
    })
}

/// Parse With attribute for fixed-size vectors
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

/// Parse `FromField` attribute for linking vector size to another field
pub fn parse_from_field_attribute(
    attr: &syn::Attribute,
    field: &mut Option<proc_macro2::Ident>,
) -> Result<(), syn::Error> {
    attr.parse_nested_meta(|meta| {
        if let Some(name) = meta.path.get_ident().cloned() {
            *field = Some(name.clone());
            Ok(())
        } else {
            Err(meta
                .error("Allowed attributes are `field_name` - Example: #[FromField(field_name)]"))
        }
    })
}
