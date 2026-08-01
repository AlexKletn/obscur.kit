#![doc = include_str!("materialized_range.doc/_module.md")]

#[doc = include_str!("materialized_range.doc/MaterializedRange.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterializedRange {
    #[doc = include_str!("materialized_range.doc/start.md")]
    start: usize,
    #[doc = include_str!("materialized_range.doc/end.md")]
    end: usize,
}

impl MaterializedRange {
    #[doc = include_str!("materialized_range.doc/new.md")]
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end: if end < start { start } else { end },
        }
    }

    #[doc = include_str!("materialized_range.doc/empty.md")]
    #[must_use]
    pub const fn empty() -> Self {
        Self { start: 0, end: 0 }
    }

    #[doc = include_str!("materialized_range.doc/start_2.md")]
    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }

    #[doc = include_str!("materialized_range.doc/end_2.md")]
    #[must_use]
    pub const fn end(self) -> usize {
        self.end
    }

    #[doc = include_str!("materialized_range.doc/len.md")]
    #[must_use]
    pub const fn len(self) -> usize {
        self.end - self.start
    }

    #[doc = include_str!("materialized_range.doc/is_empty.md")]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    #[doc = include_str!("materialized_range.doc/contains.md")]
    #[must_use]
    pub const fn contains(&self, index: usize) -> bool {
        index >= self.start && index < self.end
    }

    #[doc = include_str!("materialized_range.doc/overlaps.md")]
    #[must_use]
    pub const fn overlaps(&self, other: &MaterializedRange) -> bool {
        self.start < other.end && other.start < self.end
    }

    #[doc = include_str!("materialized_range.doc/iter.md")]
    #[must_use]
    pub const fn iter(self) -> std::ops::Range<usize> {
        self.start..self.end
    }

    #[doc = include_str!("materialized_range.doc/union.md")]
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
