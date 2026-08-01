#![doc = include_str!("layer_anchor.doc/_module.md")]

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
}
