//! Typed identifiers for known longhand properties.
//!
//! [`PropertyId`] is a *typed* enum, not a string: the cascade and computed-value
//! machinery dispatch on the variant, never on a property name. Names are kept
//! only as stable `&'static str`s for diagnostics and DevTools.
//!
//! The set of variants here is a deliberate Phase-1 starter set. The full CSS
//! longhand catalogue grows in this enum over time; adding a longhand means
//! adding a variant plus its rows in [`PropertyId::name`] and
//! [`PropertyId::default_inheritance`].

use crate::property::Inheritance;

/// A typed identifier for a known longhand CSS property.
///
/// Only longhands are represented; shorthands expand to longhands before the
/// cascade sees them. This is `Copy + Eq + Hash` so it can key maps in the
/// cascade and computed-style storage cheaply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyId {
    /// The `color` property (foreground / text color).
    Color,
    /// The `background-color` property.
    BackgroundColor,
    /// The `opacity` property.
    Opacity,
    /// The `width` property.
    Width,
    /// The `height` property.
    Height,
    /// The `display` property.
    Display,
    /// The `visibility` property.
    Visibility,
}

impl PropertyId {
    /// Returns the property's canonical CSS name as a stable string.
    ///
    /// This is intended for diagnostics and DevTools only; it is never used for
    /// dispatch (that is what the enum variant is for).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Color => "color",
            Self::BackgroundColor => "background-color",
            Self::Opacity => "opacity",
            Self::Width => "width",
            Self::Height => "height",
            Self::Display => "display",
            Self::Visibility => "visibility",
        }
    }

    /// Returns whether this property inherits from its parent by default (§4.14).
    ///
    /// This is the property's *default* behaviour; the CSS-wide keywords on
    /// individual declarations may override it per element during the cascade.
    #[must_use]
    pub const fn default_inheritance(self) -> Inheritance {
        match self {
            // Text-affecting properties inherit by default.
            Self::Color | Self::Visibility => Inheritance::Inherited,
            // Box / paint properties reset on each element.
            Self::BackgroundColor
            | Self::Opacity
            | Self::Width
            | Self::Height
            | Self::Display => Inheritance::NotInherited,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_stable() {
        assert_eq!(PropertyId::Color.name(), "color");
        assert_eq!(PropertyId::BackgroundColor.name(), "background-color");
        assert_eq!(PropertyId::Opacity.name(), "opacity");
    }

    #[test]
    fn default_inheritance_matches_css() {
        assert!(PropertyId::Color.default_inheritance().is_inherited());
        assert!(PropertyId::Visibility.default_inheritance().is_inherited());
        assert!(!PropertyId::Opacity.default_inheritance().is_inherited());
        assert!(!PropertyId::BackgroundColor.default_inheritance().is_inherited());
        assert!(!PropertyId::Display.default_inheritance().is_inherited());
    }

    #[test]
    fn is_hashable_key() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(PropertyId::Width);
        set.insert(PropertyId::Width);
        set.insert(PropertyId::Height);
        assert_eq!(set.len(), 2);
    }
}
