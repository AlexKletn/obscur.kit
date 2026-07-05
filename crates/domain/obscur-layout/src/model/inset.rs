//! The four positioning insets of a node (`top` / `right` / `bottom` / `left`).

use crate::model::Dimension;

/// A node's positioning insets — the `top` / `right` / `bottom` / `left`
/// offsets used by a positioned node (spec §4.1; CSS `inset`).
///
/// Each edge is a [`Dimension`], so it distinguishes a definite offset
/// ([`Dimension::Px`] / [`Dimension::Percent`]) from `auto`
/// ([`Dimension::Auto`], the initial value). This is separate from the pixel
/// [`Insets`](obscur_geometry::Insets) used for margin/padding/border, which
/// cannot carry `auto` or percentages.
///
/// The insets only take effect for a [`Position::Relative`](crate::model::Position::Relative)
/// (or, later, out-of-flow) node; a [`Position::Static`](crate::model::Position::Static)
/// node ignores them entirely.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Inset {
    /// The `top` inset.
    pub top: Dimension,
    /// The `right` inset.
    pub right: Dimension,
    /// The `bottom` inset.
    pub bottom: Dimension,
    /// The `left` inset.
    pub left: Dimension,
}

impl Inset {
    /// The all-`auto` insets — the CSS initial value, contributing no offset.
    pub const AUTO: Self = Self {
        top: Dimension::Auto,
        right: Dimension::Auto,
        bottom: Dimension::Auto,
        left: Dimension::Auto,
    };

    /// Builds insets from the four edges (top / right / bottom / left).
    #[must_use]
    pub const fn new(top: Dimension, right: Dimension, bottom: Dimension, left: Dimension) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl Default for Inset {
    /// The initial value: all edges `auto`.
    fn default() -> Self {
        Self::AUTO
    }
}

#[cfg(test)]
mod tests {
    use super::Inset;
    use crate::model::Dimension;

    #[test]
    fn default_is_all_auto() {
        let inset = Inset::default();
        assert!(inset.top.is_auto());
        assert!(inset.right.is_auto());
        assert!(inset.bottom.is_auto());
        assert!(inset.left.is_auto());
        assert_eq!(inset, Inset::AUTO);
    }

    #[test]
    fn new_stores_each_edge() {
        let inset = Inset::new(
            Dimension::Px(1.0),
            Dimension::Px(2.0),
            Dimension::Px(3.0),
            Dimension::Px(4.0),
        );
        assert_eq!(inset.top, Dimension::Px(1.0));
        assert_eq!(inset.right, Dimension::Px(2.0));
        assert_eq!(inset.bottom, Dimension::Px(3.0));
        assert_eq!(inset.left, Dimension::Px(4.0));
    }
}
