//! `TypeChecker`: the type-checking helper for props and simple expressions
//! (spec §2.9, §2.10).

use crate::symbol::Symbol;
use crate::typing::{ExpressionType, TypeError, ValueType};

/// The type-checking helper for props and simple expressions (spec §2.9,
/// §2.10).
///
/// Because the syntax layer captures each `.okui` expression as raw text (there
/// is no expression AST yet), this checker works on the pieces the resolution
/// and lowering steps can already produce: it classifies a *literal token's*
/// text, types a *reference* to a resolved [`Symbol`], and applies the
/// [`ValueType`] assignability algebra to check props and conditions. Full
/// expression-tree inference awaits an expression parser and is out of scope
/// here.
#[derive(Debug, Clone, Default)]
pub struct TypeChecker;

impl TypeChecker {
    /// Creates a new type checker.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Infers the type of a literal token from its raw `text` (spec §2.10).
    ///
    /// The classification is:
    ///
    /// | `text`                     | inferred type                    |
    /// |----------------------------|----------------------------------|
    /// | `true` / `false`           | constant [`Bool`](ValueType::Bool) |
    /// | `[+-]?[0-9]+`              | constant [`Int`](ValueType::Int) |
    /// | `[+-]?[0-9]+.[0-9]+`      | constant [`Float`](ValueType::Float) |
    /// | `"..."` (double-quoted)    | constant [`Text`](ValueType::Text) |
    /// | `null` / `none`            | constant `Optional<Unknown>`     |
    /// | anything else              | [`unknown`](ExpressionType::unknown) |
    ///
    /// A non-literal (for example an identifier or a compound expression) falls
    /// through to [`ExpressionType::unknown`]; its real type comes from the
    /// symbol table via [`TypeChecker::infer_reference`].
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

    /// Types a reference to a resolved `symbol` as a dynamic value of its
    /// `declared` type (spec §2.10).
    ///
    /// A reference reads a value at runtime, so the result is
    /// [`dynamic`](ExpressionType::dynamic) (never a compile-time constant). The
    /// caller supplies `declared` from the prop/state type map; the `symbol` is
    /// accepted so the signature is stable once per-symbol type lookup exists.
    #[must_use]
    pub fn infer_reference(&self, symbol: &Symbol, declared: ValueType) -> ExpressionType {
        let _ = symbol;
        ExpressionType::dynamic(declared)
    }

    /// Checks that `value` is assignable to `target`, reporting a
    /// [`Mismatch`](crate::typing::TypeErrorKind::Mismatch) at `offset`
    /// otherwise (spec §2.9).
    ///
    /// # Errors
    ///
    /// Returns a [`TypeError::mismatch`] when `value` is not assignable to
    /// `target`.
    pub fn check_assignable(
        &self,
        value: &ExpressionType,
        target: &ValueType,
        offset: u32,
    ) -> Result<(), TypeError> {
        if value.is_assignable_to(target) {
            Ok(())
        } else {
            Err(TypeError::mismatch(target.clone(), value.value_type().clone(), offset))
        }
    }

    /// Checks that `cond` may be used as a boolean condition (for example a
    /// `<If cond={..}>`), reporting a
    /// [`NotBoolean`](crate::typing::TypeErrorKind::NotBoolean) at `offset`
    /// otherwise (spec §2.9).
    ///
    /// # Errors
    ///
    /// Returns a [`TypeError::not_boolean`] when `cond` is not assignable to
    /// [`ValueType::Bool`].
    pub fn check_condition(&self, cond: &ExpressionType, offset: u32) -> Result<(), TypeError> {
        if cond.is_assignable_to(&ValueType::Bool) {
            Ok(())
        } else {
            Err(TypeError::not_boolean(cond.value_type().clone(), offset))
        }
    }
}

/// Returns whether `text` is a double-quoted string literal.
fn is_quoted_text(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() >= 2 && bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"'
}

/// Returns whether `text` is an integer literal (`[+-]?[0-9]+`).
fn is_integer(text: &str) -> bool {
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

/// Returns whether `text` is a float literal (`[+-]?[0-9]+.[0-9]+`).
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
