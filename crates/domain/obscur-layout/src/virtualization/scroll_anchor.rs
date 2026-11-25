#![doc = include_str!("../../docs/api/virtualization/scroll_anchor.doc/_module.md")]

use super::size_estimator::SizeEstimator;

#[doc = include_str!("../../docs/api/virtualization/scroll_anchor.doc/ScrollAnchor.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollAnchor {
    /// The item the viewport top is anchored to.
    index: usize,
    /// How far, in logical pixels, into the anchored item the viewport top sits
    /// (`>= 0`).
    offset_within_item: f32,
    /// The scroll offset the anchor was captured at, kept for reference.
    viewport_offset: f32,
}

impl ScrollAnchor {
    /// Creates an anchor pinned to `index`, with the viewport top
    /// `offset_within_item` into that item, captured at `viewport_offset` (spec
    /// §4.9).
    #[must_use]
    pub const fn new(index: usize, offset_within_item: f32, viewport_offset: f32) -> Self {
        Self {
            index,
            offset_within_item,
            viewport_offset,
        }
    }

    /// Creates an anchor pinned exactly to the top of `index` (both offsets `0`).
    #[must_use]
    pub const fn at_item(index: usize) -> Self {
        Self::new(index, 0.0, 0.0)
    }

    /// The anchored item index.
    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    /// How far into the anchored item the viewport top sits, in logical pixels.
    #[must_use]
    pub const fn offset_within_item(self) -> f32 {
        self.offset_within_item
    }

    /// The scroll offset this anchor was captured at.
    #[must_use]
    pub const fn viewport_offset(self) -> f32 {
        self.viewport_offset
    }

    #[doc = include_str!("../../docs/api/virtualization/scroll_anchor.doc/resolve.md")]
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
