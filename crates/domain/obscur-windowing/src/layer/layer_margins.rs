#![doc = include_str!("layer_margins.doc/_module.md")]

#[doc = include_str!("layer_margins.doc/LayerMargins.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LayerMargins {
    /// The gap from the top edge.
    pub top: i32,
    /// The gap from the right edge.
    pub right: i32,
    /// The gap from the bottom edge.
    pub bottom: i32,
    /// The gap from the left edge.
    pub left: i32,
}

impl LayerMargins {
    #[doc = include_str!("layer_margins.doc/uniform.md")]
    #[must_use]
    pub const fn uniform(px: i32) -> Self {
        Self {
            top: px,
            right: px,
            bottom: px,
            left: px,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_is_flush_against_every_edge() {
        assert_eq!(LayerMargins::default(), LayerMargins::uniform(0));
    }

    #[test]
    fn a_uniform_margin_sets_all_four_sides() {
        let margin = LayerMargins::uniform(8);
        assert_eq!(
            (margin.top, margin.right, margin.bottom, margin.left),
            (8, 8, 8, 8)
        );
    }

    #[test]
    fn a_negative_margin_is_representable() {
        // A dock sliding partly off-screen is the case this exists for.
        let margin = LayerMargins::uniform(-4);
        assert_eq!(margin.bottom, -4);
    }
}
