//! CSS length values.
//!
//! [`Length`] models the length values reachable in Phase 1: absolute pixels,
//! percentages (of some base resolved later during layout), and the `auto`
//! keyword. Font-relative units (`em`, `rem`, …), viewport units (`vw`, `vh`,
//! …) and `calc()` expressions are deliberately **not** modelled yet; they are
//! later extensions that will add variants here without disturbing existing
//! ones.

use obscur_geometry::LogicalPx;

/// A CSS length value.
///
/// Because [`Length::Px`] wraps a [`LogicalPx`] (an `f32` newtype) and
/// [`Length::Percent`] carries an `f32`, this type derives [`PartialEq`] but
/// not [`Eq`]/[`Hash`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    /// An absolute length in logical pixels.
    Px(LogicalPx),
    /// A percentage of some base dimension, expressed as a percentage value
    /// (for example `50.0` for `50%`). The base is resolved later, during
    /// layout, because it depends on context.
    Percent(f32),
    /// The `auto` keyword; its used value is resolved later, during layout.
    Auto,
}

impl Length {
    /// Convenience constructor for an absolute pixel length.
    #[must_use]
    pub const fn px(value: f32) -> Self {
        Self::Px(LogicalPx(value))
    }

    /// Returns the absolute pixel length if this is a [`Length::Px`].
    #[must_use]
    pub const fn as_px(self) -> Option<LogicalPx> {
        match self {
            Self::Px(px) => Some(px),
            _ => None,
        }
    }

    /// Returns `true` when this length is the `auto` keyword.
    #[must_use]
    pub const fn is_auto(self) -> bool {
        matches!(self, Self::Auto)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn px_constructor_wraps_logical_px() {
        assert_eq!(Length::px(12.0), Length::Px(LogicalPx(12.0)));
    }

    #[test]
    fn as_px_extracts_only_pixels() {
        assert_eq!(Length::px(4.0).as_px(), Some(LogicalPx(4.0)));
        assert_eq!(Length::Percent(50.0).as_px(), None);
        assert_eq!(Length::Auto.as_px(), None);
    }

    #[test]
    fn is_auto_detects_auto() {
        assert!(Length::Auto.is_auto());
        assert!(!Length::px(1.0).is_auto());
    }
}
