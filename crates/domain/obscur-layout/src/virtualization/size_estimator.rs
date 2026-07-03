//! The [`SizeEstimator`] — estimated and measured item extents along the scroll
//! axis (spec §4.9 *estimated geometry*).
//!
//! A virtualized list cannot afford to measure every item up front, so it starts
//! from a single `default_extent` estimate and refines it incrementally: as items
//! are really laid out, their measured extents are recorded and override the
//! estimate. The estimator maps between item indices and cumulative scroll offsets
//! by walking those extents.

use std::collections::BTreeMap;

/// Estimated and measured item extents along the scroll axis (spec §4.9
/// *estimated geometry*).
///
/// Every item is assumed to have `default_extent` logical pixels until it is
/// really measured; a measurement recorded via [`set_measured`](Self::set_measured)
/// then refines the estimate for that index. Cumulative queries
/// ([`total_extent`](Self::total_extent), [`offset_of`](Self::offset_of),
/// [`index_at_offset`](Self::index_at_offset)) walk the per-item extents so the
/// mapping stays consistent as measurements accrue.
#[derive(Debug, Clone, PartialEq)]
pub struct SizeEstimator {
    /// The assumed extent, in logical pixels, of any un-measured item.
    default_extent: f32,
    /// Known extents from real measurement, keyed by item index.
    measured: BTreeMap<usize, f32>,
}

impl SizeEstimator {
    /// Creates an estimator whose items all start at `default_extent` logical
    /// pixels, with no measurements yet (spec §4.9).
    #[must_use]
    pub fn new(default_extent: f32) -> Self {
        Self {
            default_extent,
            measured: BTreeMap::new(),
        }
    }

    /// The estimate, in logical pixels, used for any un-measured item.
    #[must_use]
    pub const fn default_extent(&self) -> f32 {
        self.default_extent
    }

    /// Records a measured `extent` (logical pixels) for the item at `index`,
    /// overriding the estimate for that index (spec §4.9).
    pub fn set_measured(&mut self, index: usize, extent: f32) {
        self.measured.insert(index, extent);
    }

    /// Forgets the measurement for `index`, returning `true` if one was present.
    pub fn clear_measured(&mut self, index: usize) -> bool {
        self.measured.remove(&index).is_some()
    }

    /// The extent of the item at `index`: its measured value if known, otherwise
    /// the default estimate (spec §4.9).
    #[must_use]
    pub fn extent(&self, index: usize) -> f32 {
        self.measured
            .get(&index)
            .copied()
            .unwrap_or(self.default_extent)
    }

    /// The total extent of all items `0..item_count` (spec §4.9).
    #[must_use]
    pub fn total_extent(&self, item_count: usize) -> f32 {
        self.offset_of(item_count, item_count)
    }

    /// The cumulative extent of every item before `index`, i.e. the scroll offset
    /// at which the item at `index` begins (spec §4.9).
    ///
    /// `index` is clamped to `item_count`, so `offset_of(item_count, ..)` is the
    /// total extent.
    #[must_use]
    pub fn offset_of(&self, index: usize, item_count: usize) -> f32 {
        let clamped = index.min(item_count);
        let mut offset = 0.0;
        let mut i = 0;
        while i < clamped {
            offset += self.extent(i);
            i += 1;
        }
        offset
    }

    /// The index of the item whose extent band contains `offset` (spec §4.9).
    ///
    /// Bands are half-open: the item at `index` spans
    /// `[offset_of(index), offset_of(index + 1))`. An `offset` at or below `0`
    /// resolves to item `0`; an `offset` at or beyond the total extent resolves to
    /// `item_count` (one past the last item). When `item_count` is `0` the result
    /// is `0`.
    #[must_use]
    pub fn index_at_offset(&self, offset: f32, item_count: usize) -> usize {
        if item_count == 0 || offset <= 0.0 {
            return 0;
        }
        let mut cumulative = 0.0;
        let mut index = 0;
        while index < item_count {
            cumulative += self.extent(index);
            if offset < cumulative {
                return index;
            }
            index += 1;
        }
        // Past the last item's band.
        item_count
    }
}

#[cfg(test)]
mod tests {
    use super::SizeEstimator;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn measured_overrides_default() {
        let mut est = SizeEstimator::new(20.0);
        assert!(approx(est.extent(0), 20.0));
        est.set_measured(0, 35.0);
        assert!(approx(est.extent(0), 35.0));
        assert!(approx(est.extent(1), 20.0));
    }

    #[test]
    fn clear_measured_reports_presence() {
        let mut est = SizeEstimator::new(20.0);
        est.set_measured(2, 50.0);
        assert!(est.clear_measured(2));
        assert!(!est.clear_measured(2));
        assert!(approx(est.extent(2), 20.0));
    }

    #[test]
    fn total_extent_sums_all_items() {
        let mut est = SizeEstimator::new(10.0);
        est.set_measured(1, 30.0);
        // Items: 10, 30, 10, 10 -> 60.
        assert!(approx(est.total_extent(4), 60.0));
    }

    #[test]
    fn offset_of_is_cumulative_and_clamped() {
        let mut est = SizeEstimator::new(10.0);
        est.set_measured(0, 40.0);
        // Before item 2: 40 + 10 = 50.
        assert!(approx(est.offset_of(2, 4), 50.0));
        // Clamped to item_count.
        assert!(approx(est.offset_of(99, 4), est.total_extent(4)));
    }

    #[test]
    fn index_at_offset_lands_in_the_right_band() {
        let mut est = SizeEstimator::new(10.0);
        est.set_measured(0, 40.0);
        // Bands: item0 [0,40), item1 [40,50), item2 [50,60), item3 [60,70).
        assert_eq!(est.index_at_offset(0.0, 4), 0);
        assert_eq!(est.index_at_offset(39.9, 4), 0);
        assert_eq!(est.index_at_offset(40.0, 4), 1);
        assert_eq!(est.index_at_offset(55.0, 4), 2);
    }

    #[test]
    fn index_at_offset_past_end_returns_item_count() {
        let est = SizeEstimator::new(10.0);
        // Total extent is 30; anything at or beyond it is past the end.
        assert_eq!(est.index_at_offset(30.0, 3), 3);
        assert_eq!(est.index_at_offset(1000.0, 3), 3);
        // Empty collection.
        assert_eq!(est.index_at_offset(5.0, 0), 0);
    }
}
