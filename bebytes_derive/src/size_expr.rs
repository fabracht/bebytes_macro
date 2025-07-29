//! Size expression parsing and evaluation
//!
//! This module handles parsing and code generation for size expressions used in
//! `#[size(expression)]` attributes. Supports mathematical operations, conditionals,
//! and field references.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, parse_str, Error, Expr, Ident, LitInt, Result, Token};

#[cfg(feature = "std")]
use std::{fmt, vec::Vec};

#[cfg(not(feature = "std"))]
use alloc::{fmt, vec::Vec};

/// A size expression that can be evaluated to determine field size
#[derive(Debug, Clone, PartialEq)]
pub enum SizeExpression {
    /// A literal integer value
    Literal(u64),
    /// A reference to another field
    FieldRef(FieldPath),
    /// Mathematical operation between two expressions
    BinaryOp {
        left: Box<SizeExpression>,
        op: BinaryOperator,
        right: Box<SizeExpression>,
    },
    /// Conditional expression: if condition { then_expr } else { else_expr }
    Conditional {
        condition: Box<Condition>,
        then_expr: Box<SizeExpression>,
        else_expr: Box<SizeExpression>,
    },
}

/// A path to a field (e.g., `count` or `header.length`)
#[derive(Debug, Clone, PartialEq)]
pub struct FieldPath {
    pub segments: Vec<Ident>,
}

/// Binary operators supported in size expressions
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %
}

/// Conditional expressions for if-else statements
#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub left: Box<SizeExpression>,
    pub op: ComparisonOperator,
    pub right: Box<SizeExpression>,
}

/// Comparison operators for conditions
#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOperator {
    Equal,              // ==
    NotEqual,           // !=
    LessThan,           // <
    LessThanOrEqual,    // <=
    GreaterThan,        // >
    GreaterThanOrEqual, // >=
}

impl SizeExpression {
    /// Parse a size expression from a string
    pub fn parse(input: &str) -> Result<Self> {
        let expr: Expr = parse_str(input)?;
        Self::from_syn_expr(&expr)
    }

    /// Convert a syn::Expr to a SizeExpression
    fn from_syn_expr(expr: &Expr) -> Result<Self> {
        match expr {
            Expr::Lit(lit) => {
                if let syn::Lit::Int(int_lit) = &lit.lit {
                    let value = int_lit.base10_parse::<u64>()?;
                    Ok(SizeExpression::Literal(value))
                } else {
                    Err(Error::new_spanned(lit, "Only integer literals are supported"))
                }
            }
            Expr::Path(path) => {
                let field_path = FieldPath::from_syn_path(&path.path)?;
                Ok(SizeExpression::FieldRef(field_path))
            }
            Expr::Binary(binary) => {
                let left = Box::new(Self::from_syn_expr(&binary.left)?);
                let right = Box::new(Self::from_syn_expr(&binary.right)?);
                let op = BinaryOperator::from_syn_binop(&binary.op)?;
                Ok(SizeExpression::BinaryOp { left, op, right })
            }
            Expr::If(if_expr) => {
                let condition = Box::new(Condition::from_syn_expr(&if_expr.cond)?);
                let then_expr = Box::new(Self::from_syn_expr(&if_expr.then_branch.stmts[0])?);
                
                let else_expr = if let Some((_, else_branch)) = &if_expr.else_branch {
                    Box::new(Self::from_syn_expr(else_branch)?)?
                } else {
                    return Err(Error::new_spanned(if_expr, "Conditional expressions must have an else clause"));
                };
                
                Ok(SizeExpression::Conditional {
                    condition,
                    then_expr,
                    else_expr,
                })
            }
            Expr::Paren(paren) => Self::from_syn_expr(&paren.expr),
            _ => Err(Error::new_spanned(
                expr,
                "Unsupported expression type in size expression",
            )),
        }
    }

    /// Generate code that evaluates this expression at runtime
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
            SizeExpression::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                let condition_code = condition.generate_condition_code();
                let then_code = then_expr.generate_evaluation_code();
                let else_code = else_expr.generate_evaluation_code();
                quote! {
                    if #condition_code {
                        #then_code
                    } else {
                        #else_code
                    }
                }
            }
        }
    }

    /// Calculate the maximum possible value this expression can evaluate to
    /// This is used for compile-time size calculation
    pub fn calculate_max_size(&self, field_types: &FieldTypeMap) -> Result<usize> {
        match self {
            SizeExpression::Literal(value) => Ok(*value as usize),
            SizeExpression::FieldRef(field_path) => {
                let field_type = field_types.get_field_type(field_path)?;
                Ok(field_type.max_value())
            }
            SizeExpression::BinaryOp { left, op, right } => {
                let left_max = left.calculate_max_size(field_types)?;
                let right_max = right.calculate_max_size(field_types)?;
                
                match op {
                    BinaryOperator::Add => Ok(left_max.saturating_add(right_max)),
                    BinaryOperator::Subtract => Ok(left_max), // Assume worst case is left operand
                    BinaryOperator::Multiply => Ok(left_max.saturating_mul(right_max)),
                    BinaryOperator::Divide => Ok(left_max), // Division reduces size
                    BinaryOperator::Modulo => Ok(right_max.saturating_sub(1)), // Modulo is < right operand
                }
            }
            SizeExpression::Conditional {
                then_expr,
                else_expr,
                ..
            } => {
                let then_max = then_expr.calculate_max_size(field_types)?;
                let else_max = else_expr.calculate_max_size(field_types)?;
                Ok(then_max.max(else_max))
            }
        }
    }

    /// Get all field references used in this expression
    pub fn get_field_references(&self) -> Vec<&FieldPath> {
        let mut refs = Vec::new();
        self.collect_field_references(&mut refs);
        refs
    }

    fn collect_field_references(&self, refs: &mut Vec<&FieldPath>) {
        match self {
            SizeExpression::FieldRef(field_path) => refs.push(field_path),
            SizeExpression::BinaryOp { left, right, .. } => {
                left.collect_field_references(refs);
                right.collect_field_references(refs);
            }
            SizeExpression::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                condition.left.collect_field_references(refs);
                condition.right.collect_field_references(refs);
                then_expr.collect_field_references(refs);
                else_expr.collect_field_references(refs);
            }
            SizeExpression::Literal(_) => {}
        }
    }
}

impl FieldPath {
    /// Create a field path from a syn::Path
    fn from_syn_path(path: &syn::Path) -> Result<Self> {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.clone())
            .collect();
        Ok(FieldPath { segments })
    }

    /// Generate code to access this field
    fn generate_access_code(&self) -> TokenStream {
        let segments = &self.segments;
        if segments.len() == 1 {
            let field = &segments[0];
            quote! { #field }
        } else {
            // For nested access like header.length
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

impl Condition {
    fn from_syn_expr(expr: &Expr) -> Result<Self> {
        if let Expr::Binary(binary) = expr {
            let left = Box::new(SizeExpression::from_syn_expr(&binary.left)?);
            let right = Box::new(SizeExpression::from_syn_expr(&binary.right)?);
            let op = ComparisonOperator::from_syn_binop(&binary.op)?;
            Ok(Condition { left, op, right })
        } else {
            Err(Error::new_spanned(expr, "Expected comparison expression"))
        }
    }

    fn generate_condition_code(&self) -> TokenStream {
        let left_code = self.left.generate_evaluation_code();
        let right_code = self.right.generate_evaluation_code();
        let op_code = self.op.generate_operator_code();
        quote! { (#left_code) #op_code (#right_code) }
    }
}

impl ComparisonOperator {
    fn from_syn_binop(op: &syn::BinOp) -> Result<Self> {
        match op {
            syn::BinOp::Eq(_) => Ok(ComparisonOperator::Equal),
            syn::BinOp::Ne(_) => Ok(ComparisonOperator::NotEqual),
            syn::BinOp::Lt(_) => Ok(ComparisonOperator::LessThan),
            syn::BinOp::Le(_) => Ok(ComparisonOperator::LessThanOrEqual),
            syn::BinOp::Gt(_) => Ok(ComparisonOperator::GreaterThan),
            syn::BinOp::Ge(_) => Ok(ComparisonOperator::GreaterThanOrEqual),
            _ => Err(Error::new_spanned(op, "Unsupported comparison operator")),
        }
    }

    fn generate_operator_code(&self) -> TokenStream {
        match self {
            ComparisonOperator::Equal => quote! { == },
            ComparisonOperator::NotEqual => quote! { != },
            ComparisonOperator::LessThan => quote! { < },
            ComparisonOperator::LessThanOrEqual => quote! { <= },
            ComparisonOperator::GreaterThan => quote! { > },
            ComparisonOperator::GreaterThanOrEqual => quote! { >= },
        }
    }
}

/// Maps field names to their types for size calculation
pub struct FieldTypeMap {
    // This will be implemented to track field types
    // For now, it's a placeholder
}

impl FieldTypeMap {
    pub fn new() -> Self {
        FieldTypeMap {}
    }

    fn get_field_type(&self, _field_path: &FieldPath) -> Result<FieldTypeInfo> {
        // TODO: Implement field type lookup
        // For now, return a default u8 type
        Ok(FieldTypeInfo::U8)
    }
}

/// Information about a field's type for size calculation
pub enum FieldTypeInfo {
    U8,
    U16,
    U32,
    U64,
    // Add more types as needed
}

impl FieldTypeInfo {
    fn max_value(&self) -> usize {
        match self {
            FieldTypeInfo::U8 => u8::MAX as usize,
            FieldTypeInfo::U16 => u16::MAX as usize,
            FieldTypeInfo::U32 => u32::MAX as usize,
            FieldTypeInfo::U64 => usize::MAX, // Cap at usize::MAX for practical purposes
        }
    }
}

impl fmt::Display for SizeExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SizeExpression::Literal(value) => write!(f, "{}", value),
            SizeExpression::FieldRef(field_path) => write!(f, "{}", field_path),
            SizeExpression::BinaryOp { left, op, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            SizeExpression::Conditional {
                condition,
                then_expr,
                else_expr,
            } => write!(f, "if {} {{ {} }} else {{ {} }}", condition, then_expr, else_expr),
        }
    }
}

impl fmt::Display for FieldPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self
            .segments
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(".");
        write!(f, "{}", path)
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Modulo => "%",
        };
        write!(f, "{}", op)
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.left, self.op, self.right)
    }
}

impl fmt::Display for ComparisonOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            ComparisonOperator::Equal => "==",
            ComparisonOperator::NotEqual => "!=",
            ComparisonOperator::LessThan => "<",
            ComparisonOperator::LessThanOrEqual => "<=",
            ComparisonOperator::GreaterThan => ">",
            ComparisonOperator::GreaterThanOrEqual => ">=",
        };
        write!(f, "{}", op)
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
            assert_eq!(*left, SizeExpression::FieldRef(FieldPath {
                segments: vec![syn::parse_str("count").unwrap()]
            }));
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
        // Just verify it generates some code - exact format may vary
        assert!(!code.is_empty());
    }
}