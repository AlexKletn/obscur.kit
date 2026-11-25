//! The CSS `box-sizing` model: whether an explicit `width` / `height` sizes the
//! content box or the border box.

#[doc = include_str!("../../docs/api/model/box_sizing.doc/BoxSizing.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoxSizing {
    /// An explicit size is the content box (the CSS initial value).
    #[default]
    ContentBox,
    /// An explicit size is the border box (`box-sizing: border-box`).
    BorderBox,
}

impl BoxSizing {
    #[doc = include_str!("../../docs/api/model/box_sizing.doc/content_extent.md")]
    #[must_use]
    pub fn content_extent(self, size: f32, bp: f32) -> f32 {
        match self {
            Self::ContentBox => size,
            Self::BorderBox => (size - bp).max(0.0),
        }
    }

    #[doc = include_str!("../../docs/api/model/box_sizing.doc/border_extent.md")]
    #[must_use]
    pub fn border_extent(self, size: f32, bp: f32) -> f32 {
        match self {
            Self::ContentBox => size + bp,
            Self::BorderBox => size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BoxSizing;

    #[test]
    fn default_is_content_box() {
        assert_eq!(BoxSizing::default(), BoxSizing::ContentBox);
    }

    #[test]
    fn content_box_passes_size_through() {
        // The authored value is already the content extent; border + padding does
        // not shrink it.
        assert_eq!(BoxSizing::ContentBox.content_extent(52.0, 16.0), 52.0);
    }

    #[test]
    fn border_box_subtracts_border_and_padding() {
        // `height: 52; padding: 8` (bp = 16) → content extent 36.
        assert_eq!(BoxSizing::BorderBox.content_extent(52.0, 16.0), 36.0);
    }

    #[test]
    fn border_box_floors_at_zero() {
        // A box smaller than its own border + padding collapses content to zero.
        assert_eq!(BoxSizing::BorderBox.content_extent(10.0, 16.0), 0.0);
    }

    #[test]
    fn zero_border_padding_is_mode_independent() {
        for mode in [BoxSizing::ContentBox, BoxSizing::BorderBox] {
            assert_eq!(mode.content_extent(100.0, 0.0), 100.0);
            assert_eq!(mode.border_extent(100.0, 0.0), 100.0);
        }
    }

    #[test]
    fn content_box_border_extent_adds_border_and_padding() {
        // A `width: 0` content-box element with a 10px horizontal border occupies
        // a 10px border box — the border widths are added, not collapsed away.
        assert_eq!(BoxSizing::ContentBox.border_extent(0.0, 10.0), 10.0);
        assert_eq!(BoxSizing::ContentBox.border_extent(52.0, 16.0), 68.0);
    }

    #[test]
    fn border_box_border_extent_passes_size_through() {
        // The authored value already IS the border box.
        assert_eq!(BoxSizing::BorderBox.border_extent(52.0, 16.0), 52.0);
    }

    #[test]
    fn border_extent_then_strip_bp_returns_authored_content() {
        // A content-box authored size mapped to its border extent and then stripped
        // of its border + padding returns the authored content size.
        let bp = 16.0;
        let authored = 52.0;
        let border = BoxSizing::ContentBox.border_extent(authored, bp);
        assert_eq!(border - bp, authored);
        // Under border-box the border extent IS the authored size, and stripping bp
        // recovers the content extent it would resolve to.
        let border_bb = BoxSizing::BorderBox.border_extent(authored, bp);
        assert_eq!(border_bb, authored);
        assert_eq!(
            BoxSizing::BorderBox.content_extent(border_bb, bp),
            authored - bp
        );
    }
}
