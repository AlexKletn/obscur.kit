//! `UiIrExpression`: a value binding in the IR — a literal or dynamic
//! reference (spec §2.5).

/// A literal value carried by a [`UiIrExpression::Literal`] (spec §2.5).
///
/// This mirrors the runtime `ViewValue`: the same closed set of scalar kinds a
/// property can hold, but as the serializable compile artifact rather than the
/// live desired-tree value. Because the [`Float`](UiIrValue::Float) variant
/// carries an `f64`, this enum derives only `Debug`, `Clone`, and `PartialEq`
/// — not `Eq`/`Hash` (floats are not totally ordered and have `NaN`).
#[derive(Debug, Clone, PartialEq)]
pub enum UiIrValue {
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

impl UiIrValue {
    /// Returns whether this value is [`Null`](UiIrValue::Null).
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

/// A value binding in the UI IR: either a literal or a dynamic reference
/// (spec §2.5).
///
/// Expressions are the serializable compile artifact a property's or event's
/// value lowers to. A [`Literal`](UiIrExpression::Literal) is a baked-in
/// constant; a [`Binding`](UiIrExpression::Binding) is a path into component
/// state or props resolved at runtime; a [`Call`](UiIrExpression::Call) applies
/// a named function or formatter to argument expressions.
///
/// Because [`UiIrValue`] contains an `f64`, this enum derives only `Debug`,
/// `Clone`, and `PartialEq`.
#[derive(Debug, Clone, PartialEq)]
pub enum UiIrExpression {
    /// A literal, baked-in value.
    Literal(UiIrValue),
    /// A path into component state or props, e.g. `"user.name"`, resolved at
    /// runtime.
    Binding(Box<str>),
    /// A function or formatter call applied to argument expressions.
    Call {
        /// The name of the function or formatter being called.
        name: Box<str>,
        /// The argument expressions, in order.
        args: Vec<UiIrExpression>,
    },
}

impl UiIrExpression {
    /// Creates a literal expression from `value`.
    #[must_use]
    pub const fn literal(value: UiIrValue) -> Self {
        Self::Literal(value)
    }

    /// Creates a literal text expression.
    #[must_use]
    pub fn text(value: impl Into<Box<str>>) -> Self {
        Self::Literal(UiIrValue::Text(value.into()))
    }

    /// Creates a literal boolean expression.
    #[must_use]
    pub const fn bool(value: bool) -> Self {
        Self::Literal(UiIrValue::Bool(value))
    }

    /// Creates a literal integer expression.
    #[must_use]
    pub const fn int(value: i64) -> Self {
        Self::Literal(UiIrValue::Int(value))
    }

    /// Creates a binding expression from a path into component state or props.
    #[must_use]
    pub fn binding(path: impl Into<Box<str>>) -> Self {
        Self::Binding(path.into())
    }

    /// Creates a call expression applying `name` to `args`.
    #[must_use]
    pub fn call(name: impl Into<Box<str>>, args: Vec<UiIrExpression>) -> Self {
        Self::Call {
            name: name.into(),
            args,
        }
    }

    /// Returns whether this expression is a [`Literal`](UiIrExpression::Literal).
    #[must_use]
    pub const fn is_literal(&self) -> bool {
        matches!(self, Self::Literal(_))
    }

    /// Returns the literal value if this expression is a
    /// [`Literal`](UiIrExpression::Literal), otherwise `None`.
    #[must_use]
    pub const fn as_literal(&self) -> Option<&UiIrValue> {
        match self {
            Self::Literal(value) => Some(value),
            Self::Binding(_) | Self::Call { .. } => None,
        }
    }

    /// Returns the binding path if this expression is a
    /// [`Binding`](UiIrExpression::Binding), otherwise `None`.
    #[must_use]
    pub fn as_binding(&self) -> Option<&str> {
        match self {
            Self::Binding(path) => Some(path),
            Self::Literal(_) | Self::Call { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_constructors_build_expected_values() {
        assert_eq!(
            UiIrExpression::text("hi"),
            UiIrExpression::Literal(UiIrValue::Text("hi".into()))
        );
        assert_eq!(
            UiIrExpression::bool(true),
            UiIrExpression::Literal(UiIrValue::Bool(true))
        );
        assert_eq!(
            UiIrExpression::int(7),
            UiIrExpression::Literal(UiIrValue::Int(7))
        );
        assert_eq!(
            UiIrExpression::literal(UiIrValue::Null),
            UiIrExpression::Literal(UiIrValue::Null)
        );
    }

    #[test]
    fn is_null_reports_null_variant() {
        assert!(UiIrValue::Null.is_null());
        assert!(!UiIrValue::Bool(false).is_null());
    }

    #[test]
    fn binding_holds_path() {
        let expr = UiIrExpression::binding("user.name");
        assert_eq!(expr, UiIrExpression::Binding("user.name".into()));
        assert_eq!(expr.as_binding(), Some("user.name"));
    }

    #[test]
    fn call_holds_name_and_args() {
        let expr = UiIrExpression::call("upper", vec![UiIrExpression::text("hi")]);
        assert_eq!(
            expr,
            UiIrExpression::Call {
                name: "upper".into(),
                args: vec![UiIrExpression::Literal(UiIrValue::Text("hi".into()))],
            }
        );
    }

    #[test]
    fn is_literal_and_as_literal() {
        let lit = UiIrExpression::int(5);
        assert!(lit.is_literal());
        assert_eq!(lit.as_literal(), Some(&UiIrValue::Int(5)));

        let bind = UiIrExpression::binding("a.b");
        assert!(!bind.is_literal());
        assert_eq!(bind.as_literal(), None);
    }

    #[test]
    fn as_binding_is_none_for_non_binding() {
        assert_eq!(UiIrExpression::int(1).as_binding(), None);
        assert_eq!(UiIrExpression::call("f", Vec::new()).as_binding(), None);
    }
}
