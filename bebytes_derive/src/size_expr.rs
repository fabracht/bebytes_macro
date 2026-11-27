use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_str, Error, Expr, Ident, Result};

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub enum SizeExpression {
    Literal(u64),
    FieldRef(FieldPath),
    BinaryOp {
        left: Box<SizeExpression>,
        op: BinaryOperator,
        right: Box<SizeExpression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldPath {
    pub segments: Vec<Ident>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

impl SizeExpression {
    pub fn parse(input: &str) -> Result<Self> {
        let expr: Expr = parse_str(input)?;
        Self::from_syn_expr(&expr)
    }

    pub fn first_unknown_field<'a>(&'a self, known: &[&Ident]) -> Option<&'a Ident> {
        match self {
            SizeExpression::Literal(_) => None,
            SizeExpression::FieldRef(path) => {
                path.segments.first().filter(|root| !known.contains(root))
            }
            SizeExpression::BinaryOp { left, right, .. } => left
                .first_unknown_field(known)
                .or_else(|| right.first_unknown_field(known)),
        }
    }

    fn from_syn_expr(expr: &Expr) -> Result<Self> {
        match expr {
            Expr::Lit(lit) => {
                if let syn::Lit::Int(int_lit) = &lit.lit {
                    let value = int_lit.base10_parse::<u64>()?;
                    Ok(SizeExpression::Literal(value))
                } else {
                    Err(Error::new_spanned(
                        lit,
                        "Only integer literals are supported",
                    ))
                }
            }
            Expr::Path(path) => {
                let field_path = FieldPath::from_syn_path(&path.path);
                Ok(SizeExpression::FieldRef(field_path))
            }
            Expr::Binary(binary) => {
                let left = Box::new(Self::from_syn_expr(&binary.left)?);
                let right = Box::new(Self::from_syn_expr(&binary.right)?);
                let op = BinaryOperator::from_syn_binop(&binary.op)?;

                if let (BinaryOperator::Divide, SizeExpression::Literal(0)) = (&op, right.as_ref())
                {
                    return Err(Error::new_spanned(
                        &binary.right,
                        "Division by zero is not allowed in size expressions",
                    ));
                }

                Ok(SizeExpression::BinaryOp { left, op, right })
            }
            Expr::Paren(paren) => Self::from_syn_expr(&paren.expr),
            _ => Err(Error::new_spanned(
                expr,
                "Unsupported expression type in size expression",
            )),
        }
    }

    pub fn generate_evaluation_code(&self) -> TokenStream {
        match self {
            SizeExpression::Literal(value) => quote! { #value as usize },
            SizeExpression::FieldRef(field_path) => {
                let field_access = field_path.generate_access_code();
                quote! { (#field_access) as usize }
            }
            SizeExpression::BinaryOp { left, op, right } => {
                let left_code = left.generate_evaluation_code();
                let right_code = right.generate_evaluation_code();
                let op_code = op.generate_operator_code();
                quote! { (#left_code) #op_code (#right_code) }
            }
        }
    }
}

impl FieldPath {
    fn from_syn_path(path: &syn::Path) -> Self {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.clone())
            .collect();
        FieldPath { segments }
    }

    fn generate_access_code(&self) -> TokenStream {
        let segments = &self.segments;
        if segments.len() == 1 {
            let field = &segments[0];
            quote! { #field }
        } else {
            let mut tokens = quote! {};
            for (i, segment) in segments.iter().enumerate() {
                if i == 0 {
                    tokens = quote! { #segment };
                } else {
                    tokens = quote! { #tokens.#segment };
                }
            }
            tokens
        }
    }
}

impl BinaryOperator {
    fn from_syn_binop(op: &syn::BinOp) -> Result<Self> {
        match op {
            syn::BinOp::Add(_) => Ok(BinaryOperator::Add),
            syn::BinOp::Sub(_) => Ok(BinaryOperator::Subtract),
            syn::BinOp::Mul(_) => Ok(BinaryOperator::Multiply),
            syn::BinOp::Div(_) => Ok(BinaryOperator::Divide),
            syn::BinOp::Rem(_) => Ok(BinaryOperator::Modulo),
            _ => Err(Error::new_spanned(op, "Unsupported binary operator")),
        }
    }

    fn generate_operator_code(&self) -> TokenStream {
        match self {
            BinaryOperator::Add => quote! { + },
            BinaryOperator::Subtract => quote! { - },
            BinaryOperator::Multiply => quote! { * },
            BinaryOperator::Divide => quote! { / },
            BinaryOperator::Modulo => quote! { % },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_literal() {
        let expr = SizeExpression::parse("42").unwrap();
        assert_eq!(expr, SizeExpression::Literal(42));
    }

    #[test]
    fn test_parse_field_reference() {
        let expr = SizeExpression::parse("count").unwrap();
        assert_eq!(
            expr,
            SizeExpression::FieldRef(FieldPath {
                segments: vec![syn::parse_str("count").unwrap()]
            })
        );
    }

    #[test]
    fn test_parse_binary_operation() {
        let expr = SizeExpression::parse("count * 4").unwrap();
        if let SizeExpression::BinaryOp { left, op, right } = expr {
            assert_eq!(
                *left,
                SizeExpression::FieldRef(FieldPath {
                    segments: vec![syn::parse_str("count").unwrap()]
                })
            );
            assert_eq!(op, BinaryOperator::Multiply);
            assert_eq!(*right, SizeExpression::Literal(4));
        } else {
            panic!("Expected binary operation");
        }
    }

    #[test]
    fn test_generate_evaluation_code() {
        let expr = SizeExpression::parse("count * 4").unwrap();
        let code = expr.generate_evaluation_code();
        assert!(!code.is_empty());
    }
}
