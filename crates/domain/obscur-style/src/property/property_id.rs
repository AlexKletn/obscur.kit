//! Typed identifiers for known longhand properties.
//!
//! [`PropertyId`] is a *typed* enum, not a string: the cascade and computed-value
//! machinery dispatch on the variant, never on a property name. Names are kept
//! only as stable `&'static str`s for diagnostics and `DevTools`.
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

    // ---- Box model: margins (Length-valued, not inherited) ----
    /// The `margin-top` longhand.
    MarginTop,
    /// The `margin-right` longhand.
    MarginRight,
    /// The `margin-bottom` longhand.
    MarginBottom,
    /// The `margin-left` longhand.
    MarginLeft,

    // ---- Box model: padding (Length-valued, not inherited) ----
    /// The `padding-top` longhand.
    PaddingTop,
    /// The `padding-right` longhand.
    PaddingRight,
    /// The `padding-bottom` longhand.
    PaddingBottom,
    /// The `padding-left` longhand.
    PaddingLeft,

    // ---- Sizing constraints (Length-valued, not inherited) ----
    /// The `min-width` property.
    MinWidth,
    /// The `min-height` property.
    MinHeight,
    /// The `max-width` property.
    MaxWidth,
    /// The `max-height` property.
    MaxHeight,

    // ---- Flex box (not inherited) ----
    /// The `gap` property (Length-valued).
    Gap,
    /// The `flex-basis` property (Length-valued).
    FlexBasis,
    /// The `flex-grow` property (Number-valued).
    FlexGrow,
    /// The `flex-shrink` property (Number-valued).
    FlexShrink,
    /// The `flex-direction` property (Keyword-valued; see
    /// [`crate::property::layout_keyword`]).
    FlexDirection,
    /// The `justify-content` property (Keyword-valued; see
    /// [`crate::property::layout_keyword`]).
    JustifyContent,
    /// The `align-items` property (Keyword-valued; see
    /// [`crate::property::layout_keyword`]).
    AlignItems,

    // ---- Typography (inherited) ----
    /// The `font-size` property (Length-valued, inherited).
    FontSize,
}

impl PropertyId {
    /// Returns the property's canonical CSS name as a stable string.
    ///
    /// This is intended for diagnostics and `DevTools` only; it is never used for
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
            Self::MarginTop => "margin-top",
            Self::MarginRight => "margin-right",
            Self::MarginBottom => "margin-bottom",
            Self::MarginLeft => "margin-left",
            Self::PaddingTop => "padding-top",
            Self::PaddingRight => "padding-right",
            Self::PaddingBottom => "padding-bottom",
            Self::PaddingLeft => "padding-left",
            Self::MinWidth => "min-width",
            Self::MinHeight => "min-height",
            Self::MaxWidth => "max-width",
            Self::MaxHeight => "max-height",
            Self::Gap => "gap",
            Self::FlexBasis => "flex-basis",
            Self::FlexGrow => "flex-grow",
            Self::FlexShrink => "flex-shrink",
            Self::FlexDirection => "flex-direction",
            Self::JustifyContent => "justify-content",
            Self::AlignItems => "align-items",
            Self::FontSize => "font-size",
        }
    }

    /// Returns whether this property inherits from its parent by default (§4.14).
    ///
    /// This is the property's *default* behaviour; the CSS-wide keywords on
    /// individual declarations may override it per element during the cascade.
    #[must_use]
    pub const fn default_inheritance(self) -> Inheritance {
        match self {
            // Text-affecting / typography properties inherit by default.
            Self::Color | Self::Visibility | Self::FontSize => Inheritance::Inherited,
            // Box / paint / layout properties reset on each element.
            Self::BackgroundColor
            | Self::Opacity
            | Self::Width
            | Self::Height
            | Self::Display
            | Self::MarginTop
            | Self::MarginRight
            | Self::MarginBottom
            | Self::MarginLeft
            | Self::PaddingTop
            | Self::PaddingRight
            | Self::PaddingBottom
            | Self::PaddingLeft
            | Self::MinWidth
            | Self::MinHeight
            | Self::MaxWidth
            | Self::MaxHeight
            | Self::Gap
            | Self::FlexBasis
            | Self::FlexGrow
            | Self::FlexShrink
            | Self::FlexDirection
            | Self::JustifyContent
            | Self::AlignItems => Inheritance::NotInherited,
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
        assert!(
            !PropertyId::BackgroundColor
                .default_inheritance()
                .is_inherited()
        );
        assert!(!PropertyId::Display.default_inheritance().is_inherited());
    }

    #[test]
    fn box_model_and_flex_names_are_canonical() {
        assert_eq!(PropertyId::MarginTop.name(), "margin-top");
        assert_eq!(PropertyId::MarginRight.name(), "margin-right");
        assert_eq!(PropertyId::MarginBottom.name(), "margin-bottom");
        assert_eq!(PropertyId::MarginLeft.name(), "margin-left");
        assert_eq!(PropertyId::PaddingTop.name(), "padding-top");
        assert_eq!(PropertyId::PaddingRight.name(), "padding-right");
        assert_eq!(PropertyId::PaddingBottom.name(), "padding-bottom");
        assert_eq!(PropertyId::PaddingLeft.name(), "padding-left");
        assert_eq!(PropertyId::MinWidth.name(), "min-width");
        assert_eq!(PropertyId::MinHeight.name(), "min-height");
        assert_eq!(PropertyId::MaxWidth.name(), "max-width");
        assert_eq!(PropertyId::MaxHeight.name(), "max-height");
        assert_eq!(PropertyId::Gap.name(), "gap");
        assert_eq!(PropertyId::FlexBasis.name(), "flex-basis");
        assert_eq!(PropertyId::FlexGrow.name(), "flex-grow");
        assert_eq!(PropertyId::FlexShrink.name(), "flex-shrink");
        assert_eq!(PropertyId::FlexDirection.name(), "flex-direction");
        assert_eq!(PropertyId::JustifyContent.name(), "justify-content");
        assert_eq!(PropertyId::AlignItems.name(), "align-items");
        assert_eq!(PropertyId::FontSize.name(), "font-size");
    }

    #[test]
    fn font_size_inherits_layout_does_not() {
        // Typography inherits, like `color`.
        assert!(PropertyId::FontSize.default_inheritance().is_inherited());
        // Box-model / flex properties reset on each element.
        for id in [
            PropertyId::MarginTop,
            PropertyId::MarginRight,
            PropertyId::MarginBottom,
            PropertyId::MarginLeft,
            PropertyId::PaddingTop,
            PropertyId::PaddingRight,
            PropertyId::PaddingBottom,
            PropertyId::PaddingLeft,
            PropertyId::MinWidth,
            PropertyId::MinHeight,
            PropertyId::MaxWidth,
            PropertyId::MaxHeight,
            PropertyId::Gap,
            PropertyId::FlexBasis,
            PropertyId::FlexGrow,
            PropertyId::FlexShrink,
            PropertyId::FlexDirection,
            PropertyId::JustifyContent,
            PropertyId::AlignItems,
        ] {
            assert!(
                !id.default_inheritance().is_inherited(),
                "{} should not inherit",
                id.name()
            );
        }
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
