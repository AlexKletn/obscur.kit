#![doc = include_str!("virtual_layout.doc/_module.md")]

use super::materialized_range::MaterializedRange;
use super::overscan_policy::OverscanPolicy;
use super::scroll_anchor::ScrollAnchor;
use super::size_estimator::SizeEstimator;

#[doc = include_str!("virtual_layout.doc/VirtualLayout.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct VirtualLayout {
    #[doc = include_str!("virtual_layout.doc/estimator.md")]
    estimator: SizeEstimator,
    #[doc = include_str!("virtual_layout.doc/overscan.md")]
    overscan: OverscanPolicy,
}

impl VirtualLayout {
    #[doc = include_str!("virtual_layout.doc/new.md")]
    #[must_use]
    pub fn new(estimator: SizeEstimator, overscan: OverscanPolicy) -> Self {
        Self {
            estimator,
            overscan,
        }
    }

    #[doc = include_str!("virtual_layout.doc/with_default_extent.md")]
    #[must_use]
    pub fn with_default_extent(default_extent: f32) -> Self {
        Self::new(SizeEstimator::new(default_extent), OverscanPolicy::none())
    }

    #[doc = include_str!("virtual_layout.doc/visible_range.md")]
    #[must_use]
    pub fn visible_range(
        &self,
        scroll_offset: f32,
        viewport_extent: f32,
        item_count: usize,
    ) -> MaterializedRange {
        if item_count == 0 {
            return MaterializedRange::empty();
        }
        let first = self.estimator.index_at_offset(scroll_offset, item_count);
        let bottom = scroll_offset + viewport_extent;
        let mut last = self.estimator.index_at_offset(bottom, item_count);
        // The viewport is half-open, `[scroll_offset, bottom)`: an item whose *top*
        // sits exactly on `bottom` has zero visible pixels and is not in the band.
        // `index_at_offset` returns that item (it "starts at" `bottom`), so step
        // back to the item straddling the last visible pixel. `offset_of(last)` is
        // `last`'s top, which is <= `bottom` by construction; equality is precisely
        // the boundary case.
        if last > first && self.estimator.offset_of(last, item_count) >= bottom {
            last -= 1;
        }
        // The visible band is the half-open range `[first, last + 1)`.
        let visible_end = last.saturating_add(1);
        let (start, end) = self.overscan.expand(first, visible_end, item_count);
        MaterializedRange::new(start, end)
    }

    #[doc = include_str!("virtual_layout.doc/anchor_at.md")]
    #[must_use]
    pub fn anchor_at(&self, scroll_offset: f32, item_count: usize) -> ScrollAnchor {
        let index = self.estimator.index_at_offset(scroll_offset, item_count);
        let offset_within_item = scroll_offset - self.estimator.offset_of(index, item_count);
        ScrollAnchor::new(index, offset_within_item, scroll_offset)
    }

    #[doc = include_str!("virtual_layout.doc/estimator_2.md")]
    #[must_use]
    pub const fn estimator(&self) -> &SizeEstimator {
        &self.estimator
    }

    #[doc = include_str!("virtual_layout.doc/estimator_mut.md")]
    pub const fn estimator_mut(&mut self) -> &mut SizeEstimator {
        &mut self.estimator
    }

    #[doc = include_str!("virtual_layout.doc/overscan_2.md")]
    #[must_use]
    pub const fn overscan(&self) -> OverscanPolicy {
        self.overscan
    }

    #[doc = include_str!("virtual_layout.doc/total_extent.md")]
    #[must_use]
    pub fn total_extent(&self, item_count: usize) -> f32 {
        self.estimator.total_extent(item_count)
    }
}

#[cfg(test)]
mod tests {
    use super::VirtualLayout;
    use crate::virtualization::overscan_policy::OverscanPolicy;
    use crate::virtualization::size_estimator::SizeEstimator;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn visible_range_uniform_items_no_overscan() {
        // 100px items, 250px viewport at offset 0 -> items 0..3.
        let layout = VirtualLayout::with_default_extent(100.0);
        let range = layout.visible_range(0.0, 250.0, 10);
        assert_eq!(range.start(), 0);
        assert_eq!(range.end(), 3);
    }

    #[test]
    fn visible_range_applies_overscan() {
        let layout = VirtualLayout::new(SizeEstimator::new(100.0), OverscanPolicy::symmetric(1));
        // Visible 0..3 widens to 0..4 (leading saturates at 0).
        let range = layout.visible_range(0.0, 250.0, 10);
        assert_eq!(range.start(), 0);
        assert_eq!(range.end(), 4);
    }

    #[test]
    fn visible_range_empty_collection() {
        let layout = VirtualLayout::with_default_extent(100.0);
        assert!(layout.visible_range(0.0, 250.0, 0).is_empty());
    }

    #[test]
    fn anchor_at_round_trips_through_resolve() {
        let layout = VirtualLayout::with_default_extent(100.0);
        // Top sits 30px into item 2 (offset 230).
        let anchor = layout.anchor_at(230.0, 10);
        assert_eq!(anchor.index(), 2);
        assert!(approx(anchor.offset_within_item(), 30.0));
        // With extents unchanged, resolving reproduces the original offset.
        assert!(approx(anchor.resolve(layout.estimator(), 10), 230.0));
    }

    #[test]
    fn total_extent_delegates_to_estimator() {
        let layout = VirtualLayout::with_default_extent(100.0);
        assert!(approx(layout.total_extent(4), 400.0));
    }
}
