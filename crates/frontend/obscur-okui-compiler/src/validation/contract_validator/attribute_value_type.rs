use obscur_okui_syntax::ast::Attribute;
use obscur_view::contract::PropTypeSurface;

/// The kind of literal an attribute was given, as far as it can be told at
/// compile time (spec §2.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AttributeValueType {
    /// A quoted `attr="…"` value, or a braced string literal `attr={"…"}`.
    String,
    /// A braced integer literal `attr={5}` (or its negation).
    Int,
    /// A braced float literal `attr={1.5}` (or its negation).
    Float,
    /// A braced boolean literal `attr={true}`.
    Bool,
    /// A binding, a computed expression, or a bare word — nothing a literal type
    /// can be read from, so it is not type-checked.
    Unknown,
}

#[doc = include_str!("../../../docs/api/validation/contract_validator.doc/attribute_value_type.md")]
pub(super) fn attribute_value_type(attribute: &Attribute) -> AttributeValueType {
    if let Some(expression) = attribute.value_expression() {
        let Ok(expr) = syn::parse_str::<syn::Expr>(&expression.text()) else {
            return AttributeValueType::Unknown;
        };
        return literal_expr_type(&expr);
    }
    if attribute.string_value().is_some() {
        return AttributeValueType::String;
    }
    AttributeValueType::Unknown
}

/// Classifies a braced expression that is a literal (or a negated numeric
/// literal), else [`Unknown`](AttributeValueType::Unknown).
fn literal_expr_type(expr: &syn::Expr) -> AttributeValueType {
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(_) => AttributeValueType::String,
            syn::Lit::Int(_) => AttributeValueType::Int,
            syn::Lit::Float(_) => AttributeValueType::Float,
            syn::Lit::Bool(_) => AttributeValueType::Bool,
            _ => AttributeValueType::Unknown,
        },
        // A negative numeric literal is `-` applied to the literal.
        syn::Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => {
            match literal_expr_type(&unary.expr) {
                ty @ (AttributeValueType::Int | AttributeValueType::Float) => ty,
                _ => AttributeValueType::Unknown,
            }
        }
        _ => AttributeValueType::Unknown,
    }
}

#[doc = include_str!("../../../docs/api/validation/contract_validator.doc/value_type_matches.md")]
pub(super) fn value_type_matches(expected: PropTypeSurface, found: AttributeValueType) -> bool {
    if found == AttributeValueType::Unknown || expected == PropTypeSurface::Other {
        return true;
    }
    match expected {
        PropTypeSurface::String => found == AttributeValueType::String,
        PropTypeSurface::Int => found == AttributeValueType::Int,
        PropTypeSurface::Float => {
            matches!(found, AttributeValueType::Float | AttributeValueType::Int)
        }
        PropTypeSurface::Bool => found == AttributeValueType::Bool,
        PropTypeSurface::Other => true,
    }
}

/// The human phrase for a declared prop type, for a diagnostic.
pub(super) fn describe_prop_type(ty: PropTypeSurface) -> &'static str {
    match ty {
        PropTypeSurface::String => "a string",
        PropTypeSurface::Int => "an integer",
        PropTypeSurface::Float => "a number",
        PropTypeSurface::Bool => "a boolean",
        PropTypeSurface::Other => "a value",
    }
}

/// The human phrase for the literal an attribute was given, for a diagnostic.
pub(super) fn describe_value_type(ty: AttributeValueType) -> &'static str {
    match ty {
        AttributeValueType::String => "a string",
        AttributeValueType::Int => "an integer",
        AttributeValueType::Float => "a number",
        AttributeValueType::Bool => "a boolean",
        AttributeValueType::Unknown => "an expression",
    }
}
