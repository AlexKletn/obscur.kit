#![doc = include_str!("expression_lowering.doc/_module.md")]

use std::str::FromStr;

use obscur_okui_syntax::ast::Expression;
use obscur_ui_ir::document::{UiIrExpression, UiIrValue};

#[doc = include_str!("expression_lowering.doc/ExpressionLowering.md")]
#[derive(Debug, Clone, Default)]
pub struct ExpressionLowering;

impl ExpressionLowering {
    #[doc = include_str!("expression_lowering.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("expression_lowering.doc/lower.md")]
    #[must_use]
    pub fn lower(&self, expr: &Expression) -> UiIrExpression {
        self.lower_text(&expr.text())
    }

    #[doc = include_str!("expression_lowering.doc/lower_text.md")]
    #[must_use]
    pub fn lower_text(&self, text: &str) -> UiIrExpression {
        let trimmed = text.trim();

        match trimmed {
            "true" => return UiIrExpression::literal(UiIrValue::Bool(true)),
            "false" => return UiIrExpression::literal(UiIrValue::Bool(false)),
            "" | "null" | "none" => return UiIrExpression::literal(UiIrValue::Null),
            _ => {}
        }

        if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
            let inner = &trimmed[1..trimmed.len() - 1];
            return UiIrExpression::literal(UiIrValue::Text(inner.into()));
        }

        // An integer literal. On overflow/parse failure, fall through.
        if let Ok(value) = i64::from_str(trimmed) {
            return UiIrExpression::literal(UiIrValue::Int(value));
        }

        // A float literal: must contain a `.` so bare identifiers and the
        // `inf`/`nan` words that `f64::from_str` also accepts do not match.
        if trimmed.contains('.')
            && let Ok(value) = f64::from_str(trimmed)
        {
            return UiIrExpression::literal(UiIrValue::Float(value));
        }

        // Anything else is treated as a binding path into component state/props.
        UiIrExpression::binding(trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lower(text: &str) -> UiIrExpression {
        ExpressionLowering::new().lower_text(text)
    }

    #[test]
    fn true_becomes_bool_literal() {
        assert_eq!(lower("true"), UiIrExpression::bool(true));
        assert_eq!(lower(" false "), UiIrExpression::bool(false));
    }

    #[test]
    fn integer_becomes_int_literal() {
        assert_eq!(lower("42"), UiIrExpression::int(42));
        assert_eq!(lower("-7"), UiIrExpression::int(-7));
    }

    #[test]
    fn decimal_becomes_float_literal() {
        assert_eq!(lower("1.5"), UiIrExpression::literal(UiIrValue::Float(1.5)));
    }

    #[test]
    fn quoted_string_becomes_text_literal() {
        assert_eq!(lower("\"hi\""), UiIrExpression::text("hi"));
    }

    #[test]
    fn dotted_path_becomes_binding() {
        assert_eq!(lower("user.name"), UiIrExpression::binding("user.name"));
    }

    #[test]
    fn empty_and_null_become_null_literal() {
        assert_eq!(lower(""), UiIrExpression::literal(UiIrValue::Null));
        assert_eq!(lower("null"), UiIrExpression::literal(UiIrValue::Null));
        assert_eq!(lower("none"), UiIrExpression::literal(UiIrValue::Null));
    }

    #[test]
    fn bare_identifier_becomes_binding() {
        assert_eq!(lower("count"), UiIrExpression::binding("count"));
    }
}
