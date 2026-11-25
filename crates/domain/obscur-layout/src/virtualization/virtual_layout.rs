#![doc = include_str!("../../docs/api/virtualization/virtual_layout.doc/_module.md")]

use super::materialized_range::MaterializedRange;
use super::overscan_policy::OverscanPolicy;
use super::scroll_anchor::ScrollAnchor;
use super::size_estimator::SizeEstimator;

#[doc = include_str!("../../docs/api/virtualization/virtual_layout.doc/VirtualLayout.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct VirtualLayout {
    /// Maps between item indices and cumulative scroll offsets.
    estimator: SizeEstimator,
    /// How many extra items to materialize beyond the viewport edges.
    overscan: OverscanPolicy,
}

impl VirtualLayout {
    /// Creates a layout from a size `estimator` and an `overscan` policy (spec
    /// §4.9).
    #[must_use]
    pub fn new(estimator: SizeEstimator, overscan: OverscanPolicy) -> Self {
        Self {
            estimator,
            overscan,
        }
    }

    /// Creates a layout with uniform `default_extent` items and no overscan.
    #[must_use]
    pub fn with_default_extent(default_extent: f32) -> Self {
        Self::new(SizeEstimator::new(default_extent), OverscanPolicy::none())
    }

    #[doc = include_str!("../../docs/api/virtualization/virtual_layout.doc/visible_range.md")]
    #[must_use]
    pub fn visible_range(
        &self,
        scroll_offset: f32,
        viewport_extent: f32,
        item_count: usize,
    ) -> MaterializedRange {
        if item_count == 0
            || !scroll_offset.is_finite()
            || !viewport_extent.is_finite()
            || viewport_extent <= 0.0
        {
            return MaterializedRange::empty();
        }
        let top_px = f64::from(scroll_offset).max(0.0);
        let bottom_px = (f64::from(scroll_offset) + f64::from(viewport_extent))
            .min(self.estimator.offset_px(item_count));
        if bottom_px <= top_px {
            return MaterializedRange::empty();
        }
        // An item touching either viewport edge contributes no visible pixels.
        let first = self
            .estimator
            .partition_item_ends(item_count, |end_px| end_px <= top_px);
        let last = self
            .estimator
            .partition_item_ends(item_count, |end_px| end_px < bottom_px);
        let visible_end = last.saturating_add(1);
        let (start, end) = self.overscan.expand(first, visible_end, item_count);
        MaterializedRange::new(start, end)
    }

    #[doc = include_str!("../../docs/api/virtualization/virtual_layout.doc/anchor_at.md")]
    #[must_use]
    pub fn anchor_at(&self, scroll_offset: f32, item_count: usize) -> ScrollAnchor {
        let index = self.estimator.index_at_offset(scroll_offset, item_count);
        let offset_within_item = scroll_offset - self.estimator.offset_of(index, item_count);
        ScrollAnchor::new(index, offset_within_item, scroll_offset)
    }

    /// The size estimator backing this layout.
    #[must_use]
    pub const fn estimator(&self) -> &SizeEstimator {
        &self.estimator
    }

    /// Mutable access to the size estimator, for recording measured extents.
    pub const fn estimator_mut(&mut self) -> &mut SizeEstimator {
        &mut self.estimator
    }

    /// The overscan policy backing this layout.
    #[must_use]
    pub const fn overscan(&self) -> OverscanPolicy {
        self.overscan
    }

    /// The total extent of all `item_count` items (delegates to the estimator).
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
    fn zero_extent_collection_has_no_visible_items_even_with_overscan() {
        let layout = VirtualLayout::new(SizeEstimator::new(0.0), OverscanPolicy::symmetric(2));
        assert!(layout.visible_range(0.0, 250.0, usize::MAX).is_empty());
    }

    #[test]
    fn empty_or_invalid_viewports_do_not_materialize_items() {
        let layout = VirtualLayout::with_default_extent(100.0);
        for viewport in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert!(layout.visible_range(0.0, viewport, 10).is_empty());
        }
        assert!(layout.visible_range(f32::NAN, 250.0, 10).is_empty());
    }

    #[test]
    fn overscan_requires_the_viewport_to_intersect_the_collection() {
        let layout = VirtualLayout::new(SizeEstimator::new(10.0), OverscanPolicy::symmetric(2));
        assert!(layout.visible_range(-20.0, 20.0, 10).is_empty());
        assert!(layout.visible_range(100.0, 20.0, 10).is_empty());
    }

    #[test]
    fn zero_extent_runs_at_the_edges_do_not_expand_the_visible_band() {
        let mut layout = VirtualLayout::with_default_extent(0.0);
        layout.estimator_mut().set_measured(500, 20.0);
        assert_eq!(layout.visible_range(0.0, 30.0, usize::MAX).iter(), 500..501);
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
