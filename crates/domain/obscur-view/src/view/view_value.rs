#![doc = include_str!("../../docs/api/view/view_value.doc/_module.md")]

#[doc = include_str!("../../docs/api/view/view_value.doc/ViewValue.md")]
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
