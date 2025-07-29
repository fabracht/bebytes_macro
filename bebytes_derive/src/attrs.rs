#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub fn parse_attributes_with_expressions(
    attributes: &[syn::Attribute],
    bits_attribute_present: &mut bool,
    errors: &mut Vec<proc_macro2::TokenStream>,
) -> (
    Option<usize>,
    Option<Vec<proc_macro2::Ident>>,
    Option<crate::size_expr::SizeExpression>,
) {
    match crate::functional::functional_attrs::parse_attributes_functional(attributes) {
        Ok(attr_data) => {
            *bits_attribute_present = attr_data.is_bits_attribute;
            (attr_data.size, attr_data.field, attr_data.size_expression)
        }
        Err(errs) => {
            for e in errs {
                errors.push(e.to_compile_error());
            }
            (None, None, None)
        }
    }
}
