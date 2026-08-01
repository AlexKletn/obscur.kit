#![doc = include_str!("scroll_anchor.doc/_module.md")]

use super::size_estimator::SizeEstimator;

#[doc = include_str!("scroll_anchor.doc/ScrollAnchor.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollAnchor {
    #[doc = include_str!("scroll_anchor.doc/index.md")]
    index: usize,
    #[doc = include_str!("scroll_anchor.doc/offset_within_item.md")]
    offset_within_item: f32,
    #[doc = include_str!("scroll_anchor.doc/viewport_offset.md")]
    viewport_offset: f32,
}

impl ScrollAnchor {
    #[doc = include_str!("scroll_anchor.doc/new.md")]
    #[must_use]
    pub const fn new(index: usize, offset_within_item: f32, viewport_offset: f32) -> Self {
        Self {
            index,
            offset_within_item,
            viewport_offset,
        }
    }

    #[doc = include_str!("scroll_anchor.doc/at_item.md")]
    #[must_use]
    pub const fn at_item(index: usize) -> Self {
        Self::new(index, 0.0, 0.0)
    }

    #[doc = include_str!("scroll_anchor.doc/index_2.md")]
    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    #[doc = include_str!("scroll_anchor.doc/offset_within_item_2.md")]
    #[must_use]
    pub const fn offset_within_item(self) -> f32 {
        self.offset_within_item
    }

    #[doc = include_str!("scroll_anchor.doc/viewport_offset_2.md")]
    #[must_use]
    pub const fn viewport_offset(self) -> f32 {
        self.viewport_offset
    }

    #[doc = include_str!("scroll_anchor.doc/resolve.md")]
    #[must_use]
    pub fn resolve(&self, estimator: &SizeEstimator, item_count: usize) -> f32 {
        estimator.offset_of(self.index, item_count) + self.offset_within_item
    }
}

#[cfg(test)]
mod tests {
    use super::ScrollAnchor;
    use crate::virtualization::size_estimator::SizeEstimator;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn at_item_zeroes_offsets() {
        let anchor = ScrollAnchor::at_item(5);
        assert_eq!(anchor.index(), 5);
        assert!(approx(anchor.offset_within_item(), 0.0));
        assert!(approx(anchor.viewport_offset(), 0.0));
    }

    #[test]
    fn resolve_keeps_anchored_item_put_when_item_above_grows() {
        let mut est = SizeEstimator::new(100.0);
        // Anchor to item 2, 10px into it. Initially offset = 200 + 10 = 210.
        let anchor = ScrollAnchor::new(2, 10.0, 210.0);
        assert!(approx(anchor.resolve(&est, 10), 210.0));

        // Item 0 grows from 100 to 160 (+60). The anchored item's resolved offset
        // shifts down by exactly that growth, keeping item 2 in the same place
        // relative to its own content.
        est.set_measured(0, 160.0);
        assert!(approx(anchor.resolve(&est, 10), 270.0));
    }
}
