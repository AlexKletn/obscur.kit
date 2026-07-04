//! `ExpressionType`: the inferred type of an expression's result (spec §2.10).

use crate::typing::ValueType;

/// The inferred type of an expression's result (spec §2.10).
///
/// Beyond the [`ValueType`], this records whether the result `is_constant` — a
/// compile-time-constant literal (for example `42` or `"hi"`) as opposed to a
/// dynamic value read from a prop or state binding. The lowering step uses that
/// flag to decide whether an expression can be folded to a constant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionType {
    value_type: ValueType,
    is_constant: bool,
}

impl ExpressionType {
    /// Creates an expression type with the given `value_type` and `is_constant`
    /// flag.
    #[must_use]
    pub const fn new(value_type: ValueType, is_constant: bool) -> Self {
        Self {
            value_type,
            is_constant,
        }
    }

    /// Creates a constant expression type of `value_type` (`is_constant` true).
    #[must_use]
    pub const fn constant(value_type: ValueType) -> Self {
        Self::new(value_type, true)
    }

    /// Creates a dynamic expression type of `value_type` (`is_constant` false).
    #[must_use]
    pub const fn dynamic(value_type: ValueType) -> Self {
        Self::new(value_type, false)
    }

    /// Creates an unknown, non-constant expression type
    /// ([`ValueType::Unknown`]).
    #[must_use]
    pub const fn unknown() -> Self {
        Self::new(ValueType::Unknown, false)
    }

    /// Returns the result's [`ValueType`].
    #[must_use]
    pub const fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    /// Returns whether the result is a compile-time constant.
    #[must_use]
    pub const fn is_constant(&self) -> bool {
        self.is_constant
    }

    /// Returns whether this result may be assigned to a slot of `target`;
    /// delegates to [`ValueType::is_assignable_to`].
    #[must_use]
    pub fn is_assignable_to(&self, target: &ValueType) -> bool {
        self.value_type.is_assignable_to(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_sets_flag_and_type() {
        let ty = ExpressionType::constant(ValueType::Int);
        assert_eq!(ty.value_type(), &ValueType::Int);
        assert!(ty.is_constant());
    }

    #[test]
    fn dynamic_is_not_constant() {
        let ty = ExpressionType::dynamic(ValueType::Text);
        assert_eq!(ty.value_type(), &ValueType::Text);
        assert!(!ty.is_constant());
    }

    #[test]
    fn unknown_is_unknown_and_not_constant() {
        let ty = ExpressionType::unknown();
        assert_eq!(ty.value_type(), &ValueType::Unknown);
        assert!(!ty.is_constant());
    }

    #[test]
    fn is_assignable_to_delegates_to_value_type() {
        let int = ExpressionType::constant(ValueType::Int);
        assert!(int.is_assignable_to(&ValueType::Float));
        assert!(!int.is_assignable_to(&ValueType::Bool));
    }
}
