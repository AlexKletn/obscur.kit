#![doc = include_str!("layer_anchor.doc/_module.md")]

use super::LayerEdge;

#[doc = include_str!("layer_anchor.doc/LayerAnchor.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LayerAnchor {
    #[doc = include_str!("layer_anchor.doc/top.md")]
    pub top: bool,
    #[doc = include_str!("layer_anchor.doc/right.md")]
    pub right: bool,
    #[doc = include_str!("layer_anchor.doc/bottom.md")]
    pub bottom: bool,
    #[doc = include_str!("layer_anchor.doc/left.md")]
    pub left: bool,
}

impl LayerAnchor {
    #[doc = include_str!("layer_anchor.doc/NONE.md")]
    pub const NONE: Self = Self {
        top: false,
        right: false,
        bottom: false,
        left: false,
    };

    #[doc = include_str!("layer_anchor.doc/TOP_BAR.md")]
    pub const TOP_BAR: Self = Self {
        top: true,
        right: true,
        bottom: false,
        left: true,
    };

    #[doc = include_str!("layer_anchor.doc/BOTTOM_BAR.md")]
    pub const BOTTOM_BAR: Self = Self {
        top: false,
        right: true,
        bottom: true,
        left: true,
    };

    #[doc = include_str!("layer_anchor.doc/LEFT_BAR.md")]
    pub const LEFT_BAR: Self = Self {
        top: true,
        right: false,
        bottom: true,
        left: true,
    };

    #[doc = include_str!("layer_anchor.doc/RIGHT_BAR.md")]
    pub const RIGHT_BAR: Self = Self {
        top: true,
        right: true,
        bottom: true,
        left: false,
    };

    #[doc = include_str!("layer_anchor.doc/ALL.md")]
    pub const ALL: Self = Self {
        top: true,
        right: true,
        bottom: true,
        left: true,
    };

    #[doc = include_str!("layer_anchor.doc/stretches_horizontally.md")]
    #[must_use]
    pub const fn stretches_horizontally(self) -> bool {
        self.left && self.right
    }

    #[doc = include_str!("layer_anchor.doc/stretches_vertically.md")]
    #[must_use]
    pub const fn stretches_vertically(self) -> bool {
        self.top && self.bottom
    }

    #[doc = include_str!("layer_anchor.doc/is_empty.md")]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        !(self.top || self.right || self.bottom || self.left)
    }

    #[doc = include_str!("layer_anchor.doc/docked_edge.md")]
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
