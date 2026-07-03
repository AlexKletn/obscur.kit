//! `ViewValue`: a minimal, **provisional** value model for Phase 1.
//!
//! # Provisional — will be superseded
//!
//! This is a deliberately tiny, closed value enum that exists only so the
//! Phase 1 vertical slice can flow concrete property values through the desired
//! view tree. It is **not** the real value system: the eventual typed value
//! model (with its own schema, coercion, and identity rules) will replace this
//! type wholesale. Treat every use of `ViewValue` as a placeholder and avoid
//! building durable behaviour on its exact shape.
//!
//! Because the `Float` variant carries an `f64`, this type derives only
//! `Debug`, `Clone`, and `PartialEq` — not `Eq`/`Hash` (floats are not totally
//! ordered and have `NaN`). Any type that transitively contains a `ViewValue`
//! inherits the same restriction.

/// A single concrete value carried by a set property in the desired view tree.
///
/// **Provisional Phase 1 placeholder** — see the module documentation. Kept as
/// plain, cloneable data with no runtime handles so the desired tree stays
/// comparable and, later, lowerable to UI IR.
#[derive(Debug, Clone, PartialEq)]
pub enum ViewValue {
    /// Absence of a value.
    Null,
    /// A boolean value.
    Bool(bool),
    /// A signed integer value.
    Int(i64),
    /// A floating-point value. Prevents deriving `Eq`/`Hash` on this enum.
    Float(f64),
    /// An owned text value.
    Text(Box<str>),
}

impl From<bool> for ViewValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for ViewValue {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<f64> for ViewValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<&str> for ViewValue {
    fn from(value: &str) -> Self {
        Self::Text(value.into())
    }
}

impl From<String> for ViewValue {
    fn from(value: String) -> Self {
        Self::Text(value.into_boxed_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions_build_expected_variants() {
        assert_eq!(ViewValue::from(true), ViewValue::Bool(true));
        assert_eq!(ViewValue::from(5i64), ViewValue::Int(5));
        assert_eq!(ViewValue::from("hi"), ViewValue::Text("hi".into()));
    }

    #[test]
    fn float_values_compare_by_partial_eq() {
        assert_eq!(ViewValue::from(1.5f64), ViewValue::Float(1.5));
        assert_ne!(ViewValue::Float(f64::NAN), ViewValue::Float(f64::NAN));
    }

    #[test]
    fn null_is_distinct() {
        assert_ne!(ViewValue::Null, ViewValue::Bool(false));
    }
}
