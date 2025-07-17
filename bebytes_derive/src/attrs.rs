
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub fn parse_attributes(
    attributes: Vec<syn::Attribute>,
    bits_attribute_present: &mut bool,
    errors: &mut Vec<proc_macro2::TokenStream>,
) -> (Option<usize>, Option<proc_macro2::Ident>) {
    match crate::functional::functional_attrs::parse_attributes_functional(&attributes) {
        Ok(attr_data) => {
            *bits_attribute_present = attr_data.is_bits_attribute;
            (attr_data.size, attr_data.field)
        }
        Err(errs) => {
            for e in errs {
                errors.push(e.to_compile_error());
            }
            (None, None)
        }
    }
}

