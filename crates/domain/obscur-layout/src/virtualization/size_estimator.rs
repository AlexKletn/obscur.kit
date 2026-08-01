#![doc = include_str!("size_estimator.doc/_module.md")]

use std::collections::BTreeMap;

#[doc = include_str!("size_estimator.doc/SizeEstimator.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct SizeEstimator {
    #[doc = include_str!("size_estimator.doc/default_extent.md")]
    default_extent: f32,
    #[doc = include_str!("size_estimator.doc/measured.md")]
    measured: BTreeMap<usize, f32>,
}

impl SizeEstimator {
    #[doc = include_str!("size_estimator.doc/new.md")]
    #[must_use]
    pub fn new(default_extent: f32) -> Self {
        Self {
            default_extent,
            measured: BTreeMap::new(),
        }
    }

    #[doc = include_str!("size_estimator.doc/default_extent_2.md")]
    #[must_use]
    pub const fn default_extent(&self) -> f32 {
        self.default_extent
    }

    #[doc = include_str!("size_estimator.doc/set_measured.md")]
    pub fn set_measured(&mut self, index: usize, extent: f32) {
        self.measured.insert(index, extent);
    }

    #[doc = include_str!("size_estimator.doc/clear_measured.md")]
    pub fn clear_measured(&mut self, index: usize) -> bool {
        self.measured.remove(&index).is_some()
    }

    #[doc = include_str!("size_estimator.doc/extent.md")]
    #[must_use]
    pub fn extent(&self, index: usize) -> f32 {
        self.measured
            .get(&index)
            .copied()
            .unwrap_or(self.default_extent)
    }

    #[doc = include_str!("size_estimator.doc/total_extent.md")]
    #[must_use]
    pub fn total_extent(&self, item_count: usize) -> f32 {
        self.offset_of(item_count, item_count)
    }

    #[doc = include_str!("size_estimator.doc/offset_of.md")]
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

    #[doc = include_str!("size_estimator.doc/index_at_offset.md")]
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
