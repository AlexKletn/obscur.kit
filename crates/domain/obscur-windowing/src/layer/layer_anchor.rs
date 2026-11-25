//! Which screen edges a layer surface is pinned to.

use super::LayerEdge;

#[doc = include_str!("../../docs/api/layer/layer_anchor.doc/LayerAnchor.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LayerAnchor {
    /// Pinned to the top edge.
    pub top: bool,
    /// Pinned to the right edge.
    pub right: bool,
    /// Pinned to the bottom edge.
    pub bottom: bool,
    /// Pinned to the left edge.
    pub left: bool,
}

impl LayerAnchor {
    /// Anchored to nothing: the compositor centres the surface.
    pub const NONE: Self = Self {
        top: false,
        right: false,
        bottom: false,
        left: false,
    };

    /// A bar across the top of the screen: full width, requested height.
    pub const TOP_BAR: Self = Self {
        top: true,
        right: true,
        bottom: false,
        left: true,
    };

    /// A bar across the bottom of the screen: full width, requested height.
    pub const BOTTOM_BAR: Self = Self {
        top: false,
        right: true,
        bottom: true,
        left: true,
    };

    /// A column down the left of the screen: full height, requested width.
    pub const LEFT_BAR: Self = Self {
        top: true,
        right: false,
        bottom: true,
        left: true,
    };

    /// A column down the right of the screen: full height, requested width.
    pub const RIGHT_BAR: Self = Self {
        top: true,
        right: true,
        bottom: true,
        left: false,
    };

    /// Every edge: the surface fills the output, and the requested size is
    /// ignored on both axes.
    pub const ALL: Self = Self {
        top: true,
        right: true,
        bottom: true,
        left: true,
    };

    #[doc = include_str!("../../docs/api/layer/layer_anchor.doc/stretches_horizontally.md")]
    #[must_use]
    pub const fn stretches_horizontally(self) -> bool {
        self.left && self.right
    }

    /// Whether the surface stretches vertically (anchored top **and** bottom).
    #[must_use]
    pub const fn stretches_vertically(self) -> bool {
        self.top && self.bottom
    }

    /// Whether no edge is anchored, so the compositor centres the surface.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        !(self.top || self.right || self.bottom || self.left)
    }

    #[doc = include_str!("../../docs/api/layer/layer_anchor.doc/docked_edge.md")]
    #[must_use]
    pub const fn docked_edge(self) -> Option<LayerEdge> {
        // Two spellings dock to each edge, and they are paired here rather than
        // listed apart: the lone edge, and the bar — three edges stretching along
        // one axis and touching one of the remaining pair, which docks to the edge
        // opposite the one it left out.
        match (self.top, self.right, self.bottom, self.left) {
            (true, false, false, false) | (true, true, false, true) => Some(LayerEdge::Top),
            (false, true, false, false) | (true, true, true, false) => Some(LayerEdge::Right),
            (false, false, true, false) | (false, true, true, true) => Some(LayerEdge::Bottom),
            (false, false, false, true) | (true, false, true, true) => Some(LayerEdge::Left),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_top_bar_stretches_across_but_not_down() {
        assert!(LayerAnchor::TOP_BAR.stretches_horizontally());
        assert!(!LayerAnchor::TOP_BAR.stretches_vertically());
    }

    #[test]
    fn a_side_bar_stretches_down_but_not_across() {
        assert!(LayerAnchor::LEFT_BAR.stretches_vertically());
        assert!(!LayerAnchor::LEFT_BAR.stretches_horizontally());
        assert!(LayerAnchor::RIGHT_BAR.stretches_vertically());
        assert!(!LayerAnchor::RIGHT_BAR.stretches_horizontally());
    }

    #[test]
    fn anchoring_to_every_edge_stretches_both_ways() {
        assert!(LayerAnchor::ALL.stretches_horizontally());
        assert!(LayerAnchor::ALL.stretches_vertically());
    }

    #[test]
    fn no_anchor_is_empty_and_stretches_neither_way() {
        assert!(LayerAnchor::NONE.is_empty());
        assert!(!LayerAnchor::NONE.stretches_horizontally());
        assert!(!LayerAnchor::NONE.stretches_vertically());
        assert!(!LayerAnchor::TOP_BAR.is_empty());
    }

    #[test]
    fn each_bar_docks_to_the_edge_it_is_named_after() {
        assert_eq!(LayerAnchor::TOP_BAR.docked_edge(), Some(LayerEdge::Top));
        assert_eq!(
            LayerAnchor::BOTTOM_BAR.docked_edge(),
            Some(LayerEdge::Bottom)
        );
        assert_eq!(LayerAnchor::LEFT_BAR.docked_edge(), Some(LayerEdge::Left));
        assert_eq!(LayerAnchor::RIGHT_BAR.docked_edge(), Some(LayerEdge::Right));
    }

    #[test]
    fn a_lone_edge_docks_to_itself() {
        let top_only = LayerAnchor {
            top: true,
            ..LayerAnchor::NONE
        };
        assert_eq!(top_only.docked_edge(), Some(LayerEdge::Top));
    }

    #[test]
    fn a_bar_docks_away_from_the_edge_it_left_out() {
        // The rule, stated as the rule rather than as four answers: for each bar,
        // the edge it omits is the opposite of the edge it docks to.
        for (bar, omitted) in [
            (LayerAnchor::TOP_BAR, LayerEdge::Bottom),
            (LayerAnchor::BOTTOM_BAR, LayerEdge::Top),
            (LayerAnchor::LEFT_BAR, LayerEdge::Right),
            (LayerAnchor::RIGHT_BAR, LayerEdge::Left),
        ] {
            assert_eq!(bar.docked_edge(), Some(omitted.opposite()), "{bar:?}");
        }
    }

    #[test]
    fn the_shapes_that_are_not_docks_say_so() {
        let corner = LayerAnchor {
            top: true,
            left: true,
            ..LayerAnchor::NONE
        };
        let band = LayerAnchor {
            left: true,
            right: true,
            ..LayerAnchor::NONE
        };
        assert_eq!(corner.docked_edge(), None, "a corner is not a dock");
        assert_eq!(band.docked_edge(), None, "a band has no edge of its own");
        assert_eq!(
            LayerAnchor::ALL.docked_edge(),
            None,
            "covering is not docking"
        );
        assert_eq!(
            LayerAnchor::NONE.docked_edge(),
            None,
            "floating is not docking"
        );
    }
}
