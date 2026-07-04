//! The typed value a property can hold.
//!
//! [`PropertyValue`] is the typed result the cascade works with — never CSS
//! source text. Keyword-valued properties (for example `display: block`) store
//! their keyword as an [`InternedSymbol`] handle, not a string: matching is done
//! on the interned identity, not by comparing characters.
//!
//! Two kinds of value are resolved *before* computed values are produced:
//!
//! * the CSS-wide keywords [`PropertyValue::Inherit`], [`PropertyValue::Initial`]
//!   and [`PropertyValue::Unset`], and
//! * an unresolved custom-property reference [`PropertyValue::CustomRef`]
//!   (`var(--name)`), which is substituted during custom-property resolution
//!   (§4.14) before computed-value resolution runs.
//!
//! `calc()` / `min()` / `max()` / `clamp()` are **not** modelled yet; they are a
//! later extension that will add a variant here.

use crate::property::{Length, StyleColor};
use obscur_identity::InternedSymbol;

/// A typed CSS property value.
///
/// Because [`PropertyValue::Number`], [`PropertyValue::Length`] and
/// [`PropertyValue::Color`] carry `f32` data, this type derives [`PartialEq`]
/// but not [`Eq`]/[`Hash`].
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    /// A keyword value, identified by its interned symbol (for example the
    /// symbols for `block`, `flex` or `hidden`). The symbol is an identity
    /// handle, not a string to be matched character-by-character.
    Keyword(InternedSymbol),
    /// A length value (see [`Length`]).
    Length(Length),
    /// A bare number (for example the value of `opacity`).
    Number(f32),
    /// A color value (see [`StyleColor`]).
    Color(StyleColor),
    /// An unresolved `var(--name)` reference to a custom property, named by its
    /// interned symbol. This is a pre-resolution placeholder: it is substituted
    /// with the referenced custom property's value before computed-value
    /// resolution (§4.14).
    CustomRef(InternedSymbol),
    /// The CSS-wide `inherit` keyword.
    Inherit,
    /// The CSS-wide `initial` keyword.
    Initial,
    /// The CSS-wide `unset` keyword (behaves as `inherit` for inherited
    /// properties and `initial` otherwise).
    Unset,
}

impl PropertyValue {
    /// Returns the color if this value is a [`PropertyValue::Color`].
    #[must_use]
    pub const fn as_color(&self) -> Option<StyleColor> {
        match self {
            Self::Color(color) => Some(*color),
            _ => None,
        }
    }

    /// Returns the length if this value is a [`PropertyValue::Length`].
    #[must_use]
    pub const fn as_length(&self) -> Option<Length> {
        match self {
            Self::Length(length) => Some(*length),
            _ => None,
        }
    }

    /// Returns `true` when this value is one of the CSS-wide keywords
    /// (`inherit`, `initial`, `unset`).
    #[must_use]
    pub const fn is_wide_keyword(&self) -> bool {
        matches!(self, Self::Inherit | Self::Initial | Self::Unset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_color_extracts_only_colors() {
        let value = PropertyValue::Color(StyleColor::BLACK);
        assert_eq!(value.as_color(), Some(StyleColor::BLACK));
        assert_eq!(PropertyValue::Number(1.0).as_color(), None);
    }

    #[test]
    fn as_length_extracts_only_lengths() {
        let value = PropertyValue::Length(Length::px(8.0));
        assert_eq!(value.as_length(), Some(Length::px(8.0)));
        assert_eq!(PropertyValue::Number(1.0).as_length(), None);
    }

    #[test]
    fn wide_keyword_detection() {
        assert!(PropertyValue::Inherit.is_wide_keyword());
        assert!(PropertyValue::Initial.is_wide_keyword());
        assert!(PropertyValue::Unset.is_wide_keyword());
        assert!(!PropertyValue::Number(0.5).is_wide_keyword());
        assert!(!PropertyValue::CustomRef(InternedSymbol::from_raw(1)).is_wide_keyword());
    }

    #[test]
    fn custom_ref_carries_symbol() {
        let name = InternedSymbol::from_raw(42);
        assert_eq!(
            PropertyValue::CustomRef(name),
            PropertyValue::CustomRef(name)
        );
    }
}
