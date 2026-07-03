//! The [`MaterializedRange`] — the half-open range of item indices currently
//! materialized (spec §4.9 *materialization*).
//!
//! A virtualized collection realizes only a window of its items; this value is
//! that window, expressed as a half-open `[start, end)` index range. It is the
//! output of the layout's visible-range computation and the input to whatever
//! actually builds the item nodes.

/// The half-open `[start, end)` range of item indices currently materialized
/// (spec §4.9 *materialization*).
///
/// `end` is never less than `start` — [`new`](Self::new) clamps an inverted range
/// to empty — so [`len`](Self::len) is always well-defined. Set operations
/// ([`contains`](Self::contains), [`overlaps`](Self::overlaps),
/// [`union`](Self::union)) treat the range as the usual half-open interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterializedRange {
    /// First materialized index (inclusive).
    start: usize,
    /// One past the last materialized index (exclusive); never below `start`.
    end: usize,
}

impl MaterializedRange {
    /// Creates a range `[start, end)`, clamping `end` up to `start` so the range
    /// is never inverted (spec §4.9).
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end: if end < start { start } else { end },
        }
    }

    /// The empty range `[0, 0)`.
    #[must_use]
    pub const fn empty() -> Self {
        Self { start: 0, end: 0 }
    }

    /// First materialized index (inclusive).
    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }

    /// One past the last materialized index (exclusive).
    #[must_use]
    pub const fn end(self) -> usize {
        self.end
    }

    /// The number of materialized items (`end - start`).
    #[must_use]
    pub const fn len(self) -> usize {
        self.end - self.start
    }

    /// Whether the range materializes no items.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Whether `index` falls within `[start, end)`.
    #[must_use]
    pub const fn contains(&self, index: usize) -> bool {
        index >= self.start && index < self.end
    }

    /// Whether this range shares at least one index with `other`.
    #[must_use]
    pub const fn overlaps(&self, other: &MaterializedRange) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// A `Range` iterator over the materialized indices (`start..end`).
    #[must_use]
    pub const fn iter(self) -> std::ops::Range<usize> {
        self.start..self.end
    }

    /// The smallest range covering both this range and `other` (min start, max
    /// end).
    #[must_use]
    pub const fn union(&self, other: &MaterializedRange) -> MaterializedRange {
        let start = if self.start < other.start {
            self.start
        } else {
            other.start
        };
        let end = if self.end > other.end {
            self.end
        } else {
            other.end
        };
        MaterializedRange::new(start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::MaterializedRange;

    #[test]
    fn len_and_is_empty() {
        let r = MaterializedRange::new(3, 8);
        assert_eq!(r.len(), 5);
        assert!(!r.is_empty());
        assert!(MaterializedRange::empty().is_empty());
    }

    #[test]
    fn inverted_range_clamps_to_empty() {
        let r = MaterializedRange::new(8, 3);
        assert_eq!(r.start(), 8);
        assert_eq!(r.end(), 8);
        assert!(r.is_empty());
    }

    #[test]
    fn contains_is_half_open() {
        let r = MaterializedRange::new(3, 8);
        assert!(!r.contains(2));
        assert!(r.contains(3));
        assert!(r.contains(7));
        assert!(!r.contains(8));
    }

    #[test]
    fn overlaps_detects_shared_indices() {
        let a = MaterializedRange::new(3, 8);
        assert!(a.overlaps(&MaterializedRange::new(7, 10)));
        // Adjacent but disjoint (half-open).
        assert!(!a.overlaps(&MaterializedRange::new(8, 10)));
        assert!(!a.overlaps(&MaterializedRange::new(0, 3)));
    }

    #[test]
    fn union_spans_both() {
        let a = MaterializedRange::new(3, 6);
        let b = MaterializedRange::new(8, 12);
        assert_eq!(a.union(&b), MaterializedRange::new(3, 12));
    }

    #[test]
    fn iter_yields_indices() {
        let r = MaterializedRange::new(2, 5);
        assert_eq!(r.iter().collect::<Vec<_>>(), vec![2, 3, 4]);
    }
}
