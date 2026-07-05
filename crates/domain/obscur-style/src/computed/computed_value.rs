//! A single, fully-resolved computed value: [`ComputedValue`].
//!
//! A [`ComputedValue`] is the end product of computed-value resolution (§4.14)
//! for one property: it is guaranteed **concrete**. Concrete means it is never a
//! CSS-wide keyword ([`PropertyValue::Inherit`], [`PropertyValue::Initial`],
//! [`PropertyValue::Unset`]) and never an unresolved custom-property reference
//! ([`PropertyValue::CustomRef`]) — those placeholders are all eliminated during
//! resolution. Only concrete leaf values (keyword / length / number / color)
//! remain.

use crate::property::{Length, LengthExpr, PropertyValue, StyleColor};
use obscur_identity::InternedSymbol;

/// A fully-resolved, concrete computed value for a single property.
///
/// The wrapped [`PropertyValue`] is invariantly concrete: it is never one of the
/// CSS-wide keywords and never a [`PropertyValue::CustomRef`]. Construct one with
/// [`ComputedValue::from_concrete`] (which enforces the invariant by returning
/// [`None`] for a non-concrete input) or with the infallible typed constructors
/// [`ComputedValue::color`], [`ComputedValue::length`], [`ComputedValue::number`]
/// and [`ComputedValue::keyword`].
///
/// Because the wrapped value may carry `f32` data, this derives [`PartialEq`] but
/// not [`Eq`]/[`Hash`].
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedValue(PropertyValue);

impl ComputedValue {
    /// Wraps a [`PropertyValue`] as a computed value, checking the concreteness
    /// invariant.
    ///
    /// Returns [`Some`] when `value` is a concrete leaf (keyword / length /
    /// number / color) and [`None`] when it is a CSS-wide keyword
    /// (`inherit` / `initial` / `unset`) or an unresolved
    /// [`PropertyValue::CustomRef`] — none of which may appear in a computed
    /// value.
    #[must_use]
    pub fn from_concrete(value: PropertyValue) -> Option<Self> {
        match value {
            PropertyValue::Keyword(_)
            | PropertyValue::Length(_)
            | PropertyValue::Calc(_)
            | PropertyValue::Number(_)
            | PropertyValue::Color(_)
            | PropertyValue::FontFamilies(_) => Some(Self(value)),
            PropertyValue::CustomRef(_)
            | PropertyValue::Inherit
            | PropertyValue::Initial
            | PropertyValue::Unset => None,
        }
    }

    /// Constructs a computed color value.
    #[must_use]
    pub const fn color(color: StyleColor) -> Self {
        Self(PropertyValue::Color(color))
    }

    /// Constructs a computed length value.
    #[must_use]
    pub const fn length(length: Length) -> Self {
        Self(PropertyValue::Length(length))
    }

    /// Constructs a computed `calc()` / `min()` / `max()` / `clamp()` value.
    #[must_use]
    pub const fn calc(expr: LengthExpr) -> Self {
        Self(PropertyValue::Calc(expr))
    }

    /// Constructs a computed bare-number value (for example `opacity`).
    #[must_use]
    pub const fn number(value: f32) -> Self {
        Self(PropertyValue::Number(value))
    }

    /// Constructs a computed keyword value from its interned symbol.
    #[must_use]
    pub const fn keyword(symbol: InternedSymbol) -> Self {
        Self(PropertyValue::Keyword(symbol))
    }

    /// Returns the underlying concrete [`PropertyValue`].
    #[must_use]
    pub const fn value(&self) -> &PropertyValue {
        &self.0
    }

    /// Returns the color if this computed value is a color.
    #[must_use]
    pub const fn as_color(&self) -> Option<StyleColor> {
        self.0.as_color()
    }

    /// Returns the length if this computed value is a length.
    #[must_use]
    pub const fn as_length(&self) -> Option<Length> {
        self.0.as_length()
    }

    /// Returns the length expression if this computed value is a `calc()` /
    /// `min()` / `max()` / `clamp()`.
    #[must_use]
    pub const fn as_calc(&self) -> Option<&LengthExpr> {
        self.0.as_calc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_concrete_accepts_leaf_values() {
        assert!(ComputedValue::from_concrete(PropertyValue::Number(1.0)).is_some());
        assert!(ComputedValue::from_concrete(PropertyValue::Color(StyleColor::BLACK)).is_some());
        assert!(ComputedValue::from_concrete(PropertyValue::Length(Length::Auto)).is_some());
        assert!(
            ComputedValue::from_concrete(PropertyValue::Keyword(InternedSymbol::from_raw(3)))
                .is_some()
        );
    }

    #[test]
    fn from_concrete_rejects_non_concrete_values() {
        assert!(ComputedValue::from_concrete(PropertyValue::Inherit).is_none());
        assert!(ComputedValue::from_concrete(PropertyValue::Initial).is_none());
        assert!(ComputedValue::from_concrete(PropertyValue::Unset).is_none());
        assert!(
            ComputedValue::from_concrete(PropertyValue::CustomRef(InternedSymbol::from_raw(1)))
                .is_none()
        );
    }

    #[test]
    fn typed_constructors_are_concrete() {
        assert_eq!(
            ComputedValue::color(StyleColor::BLACK).as_color(),
            Some(StyleColor::BLACK)
        );
        assert_eq!(
            ComputedValue::length(Length::px(4.0)).as_length(),
            Some(Length::px(4.0))
        );
        assert_eq!(
            ComputedValue::number(0.5).value(),
            &PropertyValue::Number(0.5)
        );
        let sym = InternedSymbol::from_raw(7);
        assert_eq!(
            ComputedValue::keyword(sym).value(),
            &PropertyValue::Keyword(sym)
        );
    }
}
