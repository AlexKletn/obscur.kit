#![doc = include_str!("../../docs/api/virtualization/size_estimator.doc/_module.md")]

use std::collections::BTreeMap;

use obscur_geometry::saturating_f32;

#[doc = include_str!("../../docs/api/virtualization/size_estimator.doc/SizeEstimator.md")]
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
            default_extent: valid_extent(default_extent).unwrap_or(0.0),
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
        if let Some(extent) = valid_extent(extent) {
            self.measured.insert(index, extent);
        }
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

    #[doc = include_str!("../../docs/api/virtualization/size_estimator.doc/offset_of.md")]
    #[must_use]
    pub fn offset_of(&self, index: usize, item_count: usize) -> f32 {
        saturating_f32(self.offset_px(index.min(item_count)))
    }

    pub(super) fn offset_px(&self, index: usize) -> f64 {
        let mut unmeasured = index;
        let mut measured_px = 0.0;
        for extent in self.measured.range(..index).map(|(_, extent)| extent) {
            unmeasured -= 1;
            measured_px += f64::from(*extent);
        }
        measured_px + unmeasured as f64 * f64::from(self.default_extent)
    }

    #[doc = include_str!("../../docs/api/virtualization/size_estimator.doc/index_at_offset.md")]
    #[must_use]
    pub fn index_at_offset(&self, offset: f32, item_count: usize) -> usize {
        if offset.is_nan() || offset <= 0.0 {
            return 0;
        }
        let offset_px = f64::from(offset);
        self.partition_item_ends(item_count, |end_px| end_px <= offset_px)
    }

    /// Finds the first item end outside an ordered prefix.
    pub(super) fn partition_item_ends(
        &self,
        item_count: usize,
        in_prefix: impl Fn(f64) -> bool,
    ) -> usize {
        let mut start = 0;
        let mut end = item_count;
        while start < end {
            let index = start + (end - start) / 2;
            if in_prefix(self.offset_px(index + 1)) {
                start = index + 1;
            } else {
                end = index;
            }
        }
        start
    }
}

fn valid_extent(extent_px: f32) -> Option<f32> {
    (extent_px.is_finite() && extent_px >= 0.0).then_some(extent_px)
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
    fn full_platform_item_count_has_finite_offsets_and_bounded_lookup() {
        let mut estimator = SizeEstimator::new(20.0);
        estimator.set_measured(1, 35.0);

        assert_eq!(estimator.index_at_offset(55.0, usize::MAX), 2);
        assert!(estimator.total_extent(usize::MAX).is_finite());
        assert_eq!(
            estimator.index_at_offset(f32::INFINITY, usize::MAX),
            usize::MAX
        );
    }

    #[test]
    fn invalid_measurements_do_not_replace_valid_extents() {
        let mut estimator = SizeEstimator::new(20.0);
        estimator.set_measured(1, 35.0);
        estimator.set_measured(1, -5.0);
        estimator.set_measured(1, f32::NAN);
        assert!(approx(estimator.extent(1), 35.0));
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
