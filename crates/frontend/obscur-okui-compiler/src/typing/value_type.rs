//! `ValueType`: the .okui DSL value-type vocabulary (spec §2.9).

/// A value type in the .okui DSL's type system (spec §2.9).
///
/// This is the vocabulary of types a prop, state binding or expression result
/// can carry. It is deliberately small: the DSL types are the scalar
/// primitives, homogeneous [`List`](ValueType::List)s, [`Optional`](ValueType::Optional)
/// values (spec §2.10 null/none), and the opaque [`Component`](ValueType::Component)
/// type. Two escape hatches round it out: [`Unknown`](ValueType::Unknown) marks a
/// type that has not been inferred yet, and [`Any`](ValueType::Any) matches every
/// type.
///
/// The type *algebra* lives in [`ValueType::is_assignable_to`], which the typing
/// and lowering steps use to check that a value fits where it is used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    /// A boolean (`true`/`false`).
    Bool,
    /// A signed integer.
    Int,
    /// A floating-point number.
    Float,
    /// A UTF-8 text string.
    Text,
    /// A homogeneous list whose elements all share the boxed element type.
    List(Box<ValueType>),
    /// An optional value that may be absent (`null`/`none`, spec §2.10); the box
    /// holds the wrapped (present) type.
    Optional(Box<ValueType>),
    /// A component value (spec §2.9).
    Component,
    /// A type that has not been inferred yet (a deferred placeholder).
    Unknown,
    /// The escape-hatch type that is assignable to and from every other type.
    Any,
}

impl ValueType {
    /// Builds a [`List`](ValueType::List) of `element`.
    #[must_use]
    pub fn list(element: ValueType) -> Self {
        Self::List(Box::new(element))
    }

    /// Builds an [`Optional`](ValueType::Optional) of `inner`.
    #[must_use]
    pub fn optional(inner: ValueType) -> Self {
        Self::Optional(Box::new(inner))
    }

    /// Returns whether this is a numeric type ([`Int`](ValueType::Int) or
    /// [`Float`](ValueType::Float)).
    #[must_use]
    pub const fn is_numeric(&self) -> bool {
        matches!(self, Self::Int | Self::Float)
    }

    /// Returns whether this is an [`Optional`](ValueType::Optional) type.
    #[must_use]
    pub const fn is_optional(&self) -> bool {
        matches!(self, Self::Optional(_))
    }

    /// Returns the element type of a [`List`](ValueType::List) or the wrapped
    /// type of an [`Optional`](ValueType::Optional); `None` for any other type.
    #[must_use]
    pub const fn inner(&self) -> Option<&ValueType> {
        match self {
            Self::List(element) | Self::Optional(element) => Some(element),
            _ => None,
        }
    }

    /// Returns whether a value of this type may be assigned to a slot of
    /// `target` (spec §2.9 assignability).
    ///
    /// The rules, checked in order:
    ///
    /// * [`Any`](ValueType::Any) is the escape hatch: it assigns to and from
    ///   every type, in either direction.
    /// * [`Unknown`](ValueType::Unknown) is a not-yet-inferred placeholder, so it
    ///   assigns to anything (the real check is deferred).
    /// * Identical types always assign.
    /// * [`Int`](ValueType::Int) widens to [`Float`](ValueType::Float).
    /// * A [`List`](ValueType::List)`<T>` assigns to `List<U>` exactly when its
    ///   element type `T` assigns to `U`.
    /// * An [`Optional`](ValueType::Optional)`<T>` assigns to `Optional<U>`
    ///   exactly when `T` assigns to `U`.
    /// * A bare value `T` assigns to `Optional<U>` when `T` assigns to `U` (a
    ///   present value fits an optional slot).
    /// * Otherwise the assignment is rejected.
    #[must_use]
    pub fn is_assignable_to(&self, target: &ValueType) -> bool {
        // `Any` matches everything, in both directions.
        if matches!(self, Self::Any) || matches!(target, Self::Any) {
            return true;
        }
        // A not-yet-inferred type defers the check and is accepted anywhere.
        if matches!(self, Self::Unknown) {
            return true;
        }
        // Identical types always assign.
        if self == target {
            return true;
        }
        match (self, target) {
            // Int widens to Float.
            (Self::Int, Self::Float) => true,
            // List<T> -> List<U> iff the element T -> U.
            (Self::List(from), Self::List(to)) => from.is_assignable_to(to),
            // Optional<T> -> Optional<U> iff T -> U.
            (Self::Optional(from), Self::Optional(to)) => from.is_assignable_to(to),
            // A bare value T -> Optional<U> iff T -> U (a value fits an optional).
            (_, Self::Optional(to)) => self.is_assignable_to(to),
            _ => false,
        }
    }
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool => f.write_str("Bool"),
            Self::Int => f.write_str("Int"),
            Self::Float => f.write_str("Float"),
            Self::Text => f.write_str("Text"),
            Self::List(element) => write!(f, "List<{element}>"),
            Self::Optional(inner) => write!(f, "Optional<{inner}>"),
            Self::Component => f.write_str("Component"),
            Self::Unknown => f.write_str("Unknown"),
            Self::Any => f.write_str("Any"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_wrap_element_types() {
        assert_eq!(
            ValueType::list(ValueType::Int),
            ValueType::List(Box::new(ValueType::Int))
        );
        assert_eq!(
            ValueType::optional(ValueType::Text),
            ValueType::Optional(Box::new(ValueType::Text))
        );
    }

    #[test]
    fn is_numeric_only_for_int_and_float() {
        assert!(ValueType::Int.is_numeric());
        assert!(ValueType::Float.is_numeric());
        assert!(!ValueType::Bool.is_numeric());
        assert!(!ValueType::Text.is_numeric());
    }

    #[test]
    fn is_optional_only_for_optional() {
        assert!(ValueType::optional(ValueType::Int).is_optional());
        assert!(!ValueType::Int.is_optional());
        assert!(!ValueType::list(ValueType::Int).is_optional());
    }

    #[test]
    fn inner_returns_element_of_list_and_optional() {
        assert_eq!(
            ValueType::list(ValueType::Int).inner(),
            Some(&ValueType::Int)
        );
        assert_eq!(
            ValueType::optional(ValueType::Text).inner(),
            Some(&ValueType::Text)
        );
        assert_eq!(ValueType::Int.inner(), None);
    }

    #[test]
    fn equal_types_are_assignable() {
        assert!(ValueType::Int.is_assignable_to(&ValueType::Int));
        assert!(ValueType::Text.is_assignable_to(&ValueType::Text));
    }

    #[test]
    fn int_widens_to_float_but_not_the_reverse() {
        assert!(ValueType::Int.is_assignable_to(&ValueType::Float));
        assert!(!ValueType::Float.is_assignable_to(&ValueType::Int));
    }

    #[test]
    fn value_is_assignable_to_its_optional() {
        assert!(ValueType::Int.is_assignable_to(&ValueType::optional(ValueType::Int)));
        // Int widens into an Optional<Float> too.
        assert!(ValueType::Int.is_assignable_to(&ValueType::optional(ValueType::Float)));
    }

    #[test]
    fn any_is_assignable_to_and_from_anything() {
        assert!(ValueType::Any.is_assignable_to(&ValueType::Bool));
        assert!(ValueType::Bool.is_assignable_to(&ValueType::Any));
        assert!(ValueType::Any.is_assignable_to(&ValueType::Any));
    }

    #[test]
    fn unknown_is_assignable_to_anything() {
        assert!(ValueType::Unknown.is_assignable_to(&ValueType::Bool));
        assert!(ValueType::Unknown.is_assignable_to(&ValueType::optional(ValueType::Int)));
    }

    #[test]
    fn list_covaries_over_int_to_float() {
        assert!(
            ValueType::list(ValueType::Int).is_assignable_to(&ValueType::list(ValueType::Float))
        );
        assert!(
            !ValueType::list(ValueType::Float).is_assignable_to(&ValueType::list(ValueType::Int))
        );
    }

    #[test]
    fn optional_covaries_over_its_wrapped_type() {
        assert!(
            ValueType::optional(ValueType::Int)
                .is_assignable_to(&ValueType::optional(ValueType::Float))
        );
    }

    #[test]
    fn unrelated_scalars_are_not_assignable() {
        assert!(!ValueType::Bool.is_assignable_to(&ValueType::Int));
        assert!(!ValueType::Text.is_assignable_to(&ValueType::Bool));
    }

    #[test]
    fn display_renders_nested_types() {
        assert_eq!(ValueType::Int.to_string(), "Int");
        assert_eq!(ValueType::list(ValueType::Int).to_string(), "List<Int>");
        assert_eq!(
            ValueType::optional(ValueType::list(ValueType::Text)).to_string(),
            "Optional<List<Text>>"
        );
    }
}
