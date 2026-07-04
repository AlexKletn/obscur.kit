//! `TypeError`: why a type check failed for an expression (spec §2.9).

use crate::typing::ValueType;

/// The kind of failure a [`TypeError`] reports (spec §2.9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeErrorKind {
    /// A value's type is not assignable to the expected type.
    Mismatch,
    /// Two operands cannot be compared with each other.
    NotComparable,
    /// A value used as a condition is not boolean-assignable.
    NotBoolean,
    /// A required type could not be determined.
    UnknownType,
}

/// Why a type check failed for an expression (spec §2.9).
///
/// The error records the [`kind`](TypeError::kind) of failure, the `expected`
/// and `found` [`ValueType`]s, and the byte `offset` into the source where the
/// failure was detected, so a diagnostic can point at it. This type implements
/// [`std::error::Error`] so it composes with the standard error ecosystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeError {
    kind: TypeErrorKind,
    expected: ValueType,
    found: ValueType,
    offset: u32,
}

impl TypeError {
    /// Creates a type error of `kind` with the `expected` and `found` types at
    /// byte `offset`.
    #[must_use]
    pub const fn new(
        kind: TypeErrorKind,
        expected: ValueType,
        found: ValueType,
        offset: u32,
    ) -> Self {
        Self {
            kind,
            expected,
            found,
            offset,
        }
    }

    /// Creates a [`Mismatch`](TypeErrorKind::Mismatch) error: `found` is not
    /// assignable to `expected`.
    #[must_use]
    pub const fn mismatch(expected: ValueType, found: ValueType, offset: u32) -> Self {
        Self::new(TypeErrorKind::Mismatch, expected, found, offset)
    }

    /// Creates a [`NotBoolean`](TypeErrorKind::NotBoolean) error: a condition of
    /// type `found` is not boolean-assignable.
    #[must_use]
    pub const fn not_boolean(found: ValueType, offset: u32) -> Self {
        Self::new(TypeErrorKind::NotBoolean, ValueType::Bool, found, offset)
    }

    /// Returns the error's kind.
    #[must_use]
    pub const fn kind(&self) -> &TypeErrorKind {
        &self.kind
    }

    /// Returns the expected type.
    #[must_use]
    pub const fn expected(&self) -> &ValueType {
        &self.expected
    }

    /// Returns the type that was actually found.
    #[must_use]
    pub const fn found(&self) -> &ValueType {
        &self.found
    }

    /// Returns the byte offset into the source where the failure was detected.
    #[must_use]
    pub const fn offset(&self) -> u32 {
        self.offset
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            TypeErrorKind::Mismatch => write!(
                f,
                "type mismatch: expected {}, found {} at byte {}",
                self.expected, self.found, self.offset
            ),
            TypeErrorKind::NotComparable => write!(
                f,
                "types not comparable: {} and {} at byte {}",
                self.expected, self.found, self.offset
            ),
            TypeErrorKind::NotBoolean => write!(
                f,
                "condition is not boolean: found {} at byte {}",
                self.found, self.offset
            ),
            TypeErrorKind::UnknownType => {
                write!(f, "type could not be determined at byte {}", self.offset)
            }
        }
    }
}

impl std::error::Error for TypeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mismatch_display() {
        assert_eq!(
            TypeError::mismatch(ValueType::Bool, ValueType::Int, 5).to_string(),
            "type mismatch: expected Bool, found Int at byte 5"
        );
    }

    #[test]
    fn not_boolean_display_and_expected_is_bool() {
        let err = TypeError::not_boolean(ValueType::Int, 9);
        assert_eq!(
            err.to_string(),
            "condition is not boolean: found Int at byte 9"
        );
        assert_eq!(err.expected(), &ValueType::Bool);
    }

    #[test]
    fn accessors_expose_fields() {
        let err = TypeError::mismatch(ValueType::Text, ValueType::Float, 12);
        assert_eq!(err.kind(), &TypeErrorKind::Mismatch);
        assert_eq!(err.expected(), &ValueType::Text);
        assert_eq!(err.found(), &ValueType::Float);
        assert_eq!(err.offset(), 12);
    }

    #[test]
    fn usable_as_std_error() {
        let err: &dyn std::error::Error = &TypeError::mismatch(ValueType::Bool, ValueType::Int, 0);
        assert_eq!(
            err.to_string(),
            "type mismatch: expected Bool, found Int at byte 0"
        );
    }
}
