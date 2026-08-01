#![doc = include_str!("type_checker.doc/_module.md")]

use crate::symbol::Symbol;
use crate::typing::{ExpressionType, TypeError, ValueType};

#[doc = include_str!("type_checker.doc/TypeChecker.md")]
#[derive(Debug, Clone, Default)]
pub struct TypeChecker;

impl TypeChecker {
    #[doc = include_str!("type_checker.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("type_checker.doc/infer_literal.md")]
    #[must_use]
    pub fn infer_literal(&self, text: &str) -> ExpressionType {
        match text {
            "true" | "false" => return ExpressionType::constant(ValueType::Bool),
            "null" | "none" => {
                return ExpressionType::constant(ValueType::optional(ValueType::Unknown));
            }
            _ => {}
        }
        if is_quoted_text(text) {
            ExpressionType::constant(ValueType::Text)
        } else if is_integer(text) {
            ExpressionType::constant(ValueType::Int)
        } else if is_float(text) {
            ExpressionType::constant(ValueType::Float)
        } else {
            ExpressionType::unknown()
        }
    }

    #[doc = include_str!("type_checker.doc/infer_reference.md")]
    #[must_use]
    pub fn infer_reference(&self, symbol: &Symbol, declared: ValueType) -> ExpressionType {
        let _ = symbol;
        ExpressionType::dynamic(declared)
    }

    #[doc = include_str!("type_checker.doc/check_assignable.md")]
    pub fn check_assignable(
        &self,
        value: &ExpressionType,
        target: &ValueType,
        offset: u32,
    ) -> Result<(), TypeError> {
        if value.is_assignable_to(target) {
            Ok(())
        } else {
            Err(TypeError::mismatch(
                target.clone(),
                value.value_type().clone(),
                offset,
            ))
        }
    }

    #[doc = include_str!("type_checker.doc/check_condition.md")]
    pub fn check_condition(&self, cond: &ExpressionType, offset: u32) -> Result<(), TypeError> {
        if cond.is_assignable_to(&ValueType::Bool) {
            Ok(())
        } else {
            Err(TypeError::not_boolean(cond.value_type().clone(), offset))
        }
    }
}

#[doc = include_str!("type_checker.doc/is_quoted_text.md")]
fn is_quoted_text(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() >= 2 && bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"'
}

#[doc = include_str!("type_checker.doc/is_integer.md")]
fn is_integer(text: &str) -> bool {
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

#[doc = include_str!("type_checker.doc/is_float.md")]
fn is_float(text: &str) -> bool {
    let body = text.strip_prefix(['+', '-']).unwrap_or(text);
    let Some((int_part, frac_part)) = body.split_once('.') else {
        return false;
    };
    !int_part.is_empty()
        && !frac_part.is_empty()
        && int_part.bytes().all(|b| b.is_ascii_digit())
        && frac_part.bytes().all(|b| b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::Symbol;
    use crate::typing::TypeErrorKind;

    #[test]
    fn infer_literal_classifies_each_form() {
        let tc = TypeChecker::new();

        let b = tc.infer_literal("true");
        assert_eq!(b.value_type(), &ValueType::Bool);
        assert!(b.is_constant());
        assert_eq!(tc.infer_literal("false").value_type(), &ValueType::Bool);

        let i = tc.infer_literal("42");
        assert_eq!(i.value_type(), &ValueType::Int);
        assert!(i.is_constant());
        assert_eq!(tc.infer_literal("-7").value_type(), &ValueType::Int);

        let fl = tc.infer_literal("1.5");
        assert_eq!(fl.value_type(), &ValueType::Float);
        assert!(fl.is_constant());
        assert_eq!(tc.infer_literal("-0.25").value_type(), &ValueType::Float);

        let t = tc.infer_literal("\"hi\"");
        assert_eq!(t.value_type(), &ValueType::Text);
        assert!(t.is_constant());

        let n = tc.infer_literal("null");
        assert_eq!(n.value_type(), &ValueType::optional(ValueType::Unknown));
        assert!(n.is_constant());
        assert_eq!(
            tc.infer_literal("none").value_type(),
            &ValueType::optional(ValueType::Unknown)
        );
    }

    #[test]
    fn infer_literal_falls_through_to_unknown() {
        let tc = TypeChecker::new();
        let u = tc.infer_literal("foo");
        assert_eq!(u.value_type(), &ValueType::Unknown);
        assert!(!u.is_constant());
        // A bare `.` is not a float.
        assert_eq!(tc.infer_literal(".").value_type(), &ValueType::Unknown);
        assert_eq!(tc.infer_literal("12x").value_type(), &ValueType::Unknown);
    }

    #[test]
    fn infer_reference_is_dynamic_of_declared() {
        let tc = TypeChecker::new();
        let sym = Symbol::prop("count");
        let r = tc.infer_reference(&sym, ValueType::Int);
        assert_eq!(r.value_type(), &ValueType::Int);
        assert!(!r.is_constant());
    }

    #[test]
    fn check_assignable_ok_and_mismatch() {
        let tc = TypeChecker::new();
        let int = ExpressionType::constant(ValueType::Int);
        assert!(tc.check_assignable(&int, &ValueType::Float, 0).is_ok());

        let err = tc.check_assignable(&int, &ValueType::Bool, 3).unwrap_err();
        assert_eq!(err.kind(), &TypeErrorKind::Mismatch);
        assert_eq!(err.expected(), &ValueType::Bool);
        assert_eq!(err.found(), &ValueType::Int);
        assert_eq!(err.offset(), 3);
    }

    #[test]
    fn check_condition_accepts_bool_rejects_int() {
        let tc = TypeChecker::new();
        let cond = ExpressionType::constant(ValueType::Bool);
        assert!(tc.check_condition(&cond, 0).is_ok());

        let bad = ExpressionType::constant(ValueType::Int);
        let err = tc.check_condition(&bad, 6).unwrap_err();
        assert_eq!(err.kind(), &TypeErrorKind::NotBoolean);
        assert_eq!(err.found(), &ValueType::Int);
        assert_eq!(err.offset(), 6);
    }
}
