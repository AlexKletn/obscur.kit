//! The [`TimelineId`] stable timeline identity (spec §4.17 "stable identity").

#[doc = include_str!("../../docs/api/timeline/timeline_id.doc/TimelineId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimelineId(u64);

impl TimelineId {
    /// Wraps a raw handle value into a `TimelineId`.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw handle value backing this identifier.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_raw_value() {
        assert_eq!(TimelineId::from_raw(42).raw(), 42);
        assert_eq!(TimelineId::from_raw(0).raw(), 0);
        assert_eq!(TimelineId::from_raw(u64::MAX).raw(), u64::MAX);
    }

    #[test]
    fn equal_ids_compare_equal() {
        assert_eq!(TimelineId::from_raw(7), TimelineId::from_raw(7));
        assert_ne!(TimelineId::from_raw(7), TimelineId::from_raw(8));
    }

    #[test]
    fn orders_by_raw_value() {
        assert!(TimelineId::from_raw(1) < TimelineId::from_raw(2));
    }
}
