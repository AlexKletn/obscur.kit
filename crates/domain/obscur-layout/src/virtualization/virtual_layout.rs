//! The [`VirtualLayout`] — computes which items to materialize for a scroll
//! position (spec §4.9 *virtualization and collections*).
//!
//! This is the reusable 1-D virtualization mechanism the higher-level collections
//! (tables, trees, grids) compose: given a scroll offset and viewport extent it
//! resolves the visible item band via the [`SizeEstimator`], widens it by the
//! [`OverscanPolicy`], and clamps the result to a [`MaterializedRange`]. It also
//! captures [`ScrollAnchor`]s for stable scroll positions across resizes.

use super::materialized_range::MaterializedRange;
use super::overscan_policy::OverscanPolicy;
use super::scroll_anchor::ScrollAnchor;
use super::size_estimator::SizeEstimator;

/// Computes the materialized item range for a scroll position (spec §4.9).
///
/// A `VirtualLayout` pairs a [`SizeEstimator`] (the index ↔ offset mapping,
/// refined by measurement) with an [`OverscanPolicy`] (how far past the viewport
/// to materialize). This is the reusable virtualization mechanism; tables, trees
/// and grids build on it rather than reimplementing materialization.
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

    /// The materialized range for a viewport of `viewport_extent` logical pixels
    /// starting at `scroll_offset`, over `item_count` items (spec §4.9).
    ///
    /// The visible band runs from the item at `scroll_offset` through the item at
    /// `scroll_offset + viewport_extent`, i.e. `[first, last + 1)`. That band is
    /// then widened by the overscan policy and clamped to `[0, item_count]`. An
    /// empty collection yields an empty range.
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
        let last = self
            .estimator
            .index_at_offset(scroll_offset + viewport_extent, item_count);
        // `last` is the item containing the viewport bottom; include it, so the
        // visible band is the half-open range `[first, last + 1)`.
        let visible_end = last.saturating_add(1);
        let (start, end) = self.overscan.expand(first, visible_end, item_count);
        MaterializedRange::new(start, end)
    }

    /// Captures a [`ScrollAnchor`] at the current viewport top (spec §4.9).
    ///
    /// The anchor pins the item at `scroll_offset`, remembering how far into that
    /// item the top sits, so the scroll position can be restored after items above
    /// it are inserted or re-measured.
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
