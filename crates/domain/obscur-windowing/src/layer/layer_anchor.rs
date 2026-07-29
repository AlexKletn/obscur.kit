//! Which screen edges a layer surface is pinned to (spec §6.5).

/// The set of edges a layer surface is anchored to.
///
/// Anchoring does two things at once, and conflating them is the usual source of
/// surprise:
///
/// 1. it decides **where** the surface sits, and
/// 2. anchoring to two opposite edges makes the surface **stretch** between
///    them, overriding the size requested on that axis.
///
/// So a bar across the top is `top | left | right` — anchored to three edges, it
/// spans the full width and takes its height from the requested size. Anchored
/// to `top` alone it would be centred horizontally at whatever width it asked
/// for.
///
/// Anchored to nothing, the compositor centres the surface.
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

    /// Whether the surface stretches horizontally, because it is anchored to
    /// both the left and right edges.
    ///
    /// When this holds, the width in [`LayerSurfaceConfig`](super::LayerSurfaceConfig)
    /// is not what the surface gets — the compositor decides it. Reading the
    /// configure event, rather than the request, is the only correct way to know
    /// the real size.
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
