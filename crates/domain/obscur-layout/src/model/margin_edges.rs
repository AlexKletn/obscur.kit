//! A box's four margin edges.

use obscur_geometry::Insets;

use crate::model::Margin;

#[doc = include_str!("../../docs/api/model/margin_edges.doc/MarginEdges.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MarginEdges {
    /// The top edge.
    pub top: Margin,
    /// The right edge.
    pub right: Margin,
    /// The bottom edge.
    pub bottom: Margin,
    /// The left edge.
    pub left: Margin,
}

impl MarginEdges {
    /// Edges that are all zero — the initial value.
    pub const ZERO: Self = Self {
        top: Margin::ZERO,
        right: Margin::ZERO,
        bottom: Margin::ZERO,
        left: Margin::ZERO,
    };

    /// Edges built from four lengths, none of them `auto`.
    #[must_use]
    pub const fn px(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top: Margin::Px(top),
            right: Margin::Px(right),
            bottom: Margin::Px(bottom),
            left: Margin::Px(left),
        }
    }

    /// The same margin on all four edges.
    #[must_use]
    pub const fn uniform(value: f32) -> Self {
        Self::px(value, value, value, value)
    }

    #[doc = include_str!("../../docs/api/model/margin_edges.doc/resolve.md")]
    #[must_use]
    pub fn resolve(&self, basis: f32) -> Insets {
        Insets {
            top: self.top.resolve(basis),
            right: self.right.resolve(basis),
            bottom: self.bottom.resolve(basis),
            left: self.left.resolve(basis),
        }
    }

    /// Whether either **inline**-axis edge is `auto` — the pair that centres a box
    /// (`margin: 0 auto`).
    #[must_use]
    pub const fn has_auto_inline(&self) -> bool {
        self.left.is_auto() || self.right.is_auto()
    }
}

#[cfg(test)]
mod tests {
    use super::MarginEdges;
    use crate::model::Margin;
    use obscur_geometry::Insets;

    #[test]
    fn zero_is_the_default() {
        assert_eq!(MarginEdges::default(), MarginEdges::ZERO);
        assert_eq!(MarginEdges::ZERO.resolve(0.0), Insets::uniform(0.0));
        assert!(!MarginEdges::ZERO.has_auto_inline());
    }

    #[test]
    fn px_and_uniform_build_length_edges_in_css_order() {
        let edges = MarginEdges::px(1.0, 2.0, 3.0, 4.0);
        assert_eq!(edges.top, Margin::Px(1.0));
        assert_eq!(edges.right, Margin::Px(2.0));
        assert_eq!(edges.bottom, Margin::Px(3.0));
        assert_eq!(edges.left, Margin::Px(4.0));
        assert_eq!(MarginEdges::uniform(5.0).resolve(0.0), Insets::uniform(5.0));
    }

    #[test]
    fn resolve_flattens_auto_edges_to_lengths() {
        let centred = MarginEdges {
            left: Margin::Auto,
            right: Margin::Auto,
            ..MarginEdges::px(10.0, 0.0, 20.0, 0.0)
        };
        assert_eq!(
            centred.resolve(0.0),
            Insets {
                top: 10.0,
                right: 0.0,
                bottom: 20.0,
                left: 0.0
            },
            "the block edges survive; the `auto` inline ones read as zero"
        );
    }

    #[test]
    fn has_auto_inline_spots_either_inline_edge_and_ignores_the_block_ones() {
        let both = MarginEdges {
            left: Margin::Auto,
            right: Margin::Auto,
            ..MarginEdges::ZERO
        };
        assert!(both.has_auto_inline(), "`margin: 0 auto`");

        let one = MarginEdges {
            left: Margin::Auto,
            ..MarginEdges::ZERO
        };
        assert!(one.has_auto_inline(), "a single `auto` still distributes");

        // A block-axis `auto` does not centre anything in normal flow (CSS 2.1
        // §10.6.3 makes it zero), so it must not put the inline solver to work.
        let block_only = MarginEdges {
            top: Margin::Auto,
            bottom: Margin::Auto,
            ..MarginEdges::ZERO
        };
        assert!(!block_only.has_auto_inline());
    }
}
