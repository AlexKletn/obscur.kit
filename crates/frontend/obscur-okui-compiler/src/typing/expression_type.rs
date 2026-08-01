#![doc = include_str!("expression_type.doc/_module.md")]

use crate::typing::ValueType;

#[doc = include_str!("expression_type.doc/ExpressionType.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionType {
    value_type: ValueType,
    is_constant: bool,
}

impl ExpressionType {
    #[doc = include_str!("expression_type.doc/new.md")]
    #[must_use]
    pub const fn new(value_type: ValueType, is_constant: bool) -> Self {
        Self {
            value_type,
            is_constant,
        }
    }

    #[doc = include_str!("expression_type.doc/constant.md")]
    #[must_use]
    pub const fn constant(value_type: ValueType) -> Self {
        Self::new(value_type, true)
    }

    #[doc = include_str!("expression_type.doc/dynamic.md")]
    #[must_use]
    pub const fn dynamic(value_type: ValueType) -> Self {
        Self::new(value_type, false)
    }

    #[doc = include_str!("expression_type.doc/unknown.md")]
    #[must_use]
    pub const fn unknown() -> Self {
        Self::new(ValueType::Unknown, false)
    }

    #[doc = include_str!("expression_type.doc/value_type.md")]
    #[must_use]
    pub const fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    #[doc = include_str!("expression_type.doc/is_constant.md")]
    #[must_use]
    pub const fn is_constant(&self) -> bool {
        self.is_constant
    }

    #[doc = include_str!("expression_type.doc/is_assignable_to.md")]
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
