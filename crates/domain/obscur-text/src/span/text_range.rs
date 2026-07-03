//! A half-open byte range into a UTF-8 text buffer (spec §5.10).

/// A half-open byte range `[start, end)` into a UTF-8 text buffer (spec §5.10).
///
/// Rich text is addressed by byte offsets, not char or grapheme indices: a
/// `TextRange` names the bytes `start..end` of some text buffer. Callers are
/// responsible for keeping `start` and `end` on UTF-8 character boundaries; this
/// type performs no boundary validation and treats offsets as opaque `usize`
/// positions.
///
/// The range is always normalized so that `start <= end`. Constructors that
/// receive an inverted range clamp `end` up to `start` (see [`TextRange::new`]),
/// so `len` never underflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TextRange {
    /// The inclusive start byte offset.
    pub start: usize,
    /// The exclusive end byte offset; always `>= start`.
    pub end: usize,
}

impl TextRange {
    /// Creates a range spanning `start..end`.
    ///
    /// The range is normalized: if `end < start` the end is clamped up to
    /// `start` (`end.max(start)`), yielding an empty range at `start` rather
    /// than an inverted one. A `debug_assert` flags inverted input in debug
    /// builds.
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        debug_assert!(start <= end, "TextRange::new called with end < start");
        Self {
            start,
            end: if end > start { end } else { start },
        }
    }

    /// Creates an empty (collapsed) range positioned at `at`, where
    /// `start == end == at`.
    #[must_use]
    pub const fn empty(at: usize) -> Self {
        Self { start: at, end: at }
    }

    /// Returns the length of the range in bytes (`end - start`).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end - self.start
    }

    /// Returns whether the range is empty (`start == end`).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Returns whether `offset` lies within the range (`start <= offset < end`).
    ///
    /// An empty range contains no offset.
    #[must_use]
    pub const fn contains(&self, offset: usize) -> bool {
        self.start <= offset && offset < self.end
    }

    /// Returns whether this range fully contains `other`.
    ///
    /// An empty `other` is contained when its position lies within
    /// `start..=end`.
    #[must_use]
    pub const fn contains_range(&self, other: TextRange) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    /// Returns whether this range and `other` share at least one byte.
    ///
    /// Ranges that merely touch at an endpoint (for example `0..2` and `2..4`)
    /// do not intersect, and empty ranges never intersect.
    #[must_use]
    pub const fn intersects(&self, other: TextRange) -> bool {
        let start = if self.start > other.start {
            self.start
        } else {
            other.start
        };
        let end = if self.end < other.end {
            self.end
        } else {
            other.end
        };
        start < end
    }

    /// Returns the overlapping range shared with `other`, or `None` when they do
    /// not intersect (spec §5.10).
    #[must_use]
    pub const fn intersection(&self, other: TextRange) -> Option<TextRange> {
        let start = if self.start > other.start {
            self.start
        } else {
            other.start
        };
        let end = if self.end < other.end {
            self.end
        } else {
            other.end
        };
        if start < end {
            Some(TextRange { start, end })
        } else {
            None
        }
    }

    /// Returns the smallest range covering both this range and `other`: the
    /// minimum start and maximum end (spec §5.10).
    #[must_use]
    pub const fn union(&self, other: TextRange) -> TextRange {
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
        TextRange { start, end }
    }

    /// Returns this range translated by `delta` bytes, saturating at the bounds
    /// of `usize` so neither endpoint overflows or underflows.
    #[must_use]
    pub const fn shifted(&self, delta: isize) -> TextRange {
        TextRange {
            start: self.start.saturating_add_signed(delta),
            end: self.end.saturating_add_signed(delta),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn len_and_is_empty() {
        assert_eq!(TextRange::new(2, 7).len(), 5);
        assert!(!TextRange::new(2, 7).is_empty());
        assert_eq!(TextRange::empty(4).len(), 0);
        assert!(TextRange::empty(4).is_empty());
    }

    #[test]
    fn new_normalizes_endpoints() {
        // Equal endpoints are the boundary case and stay collapsed. Inverted
        // input (`end < start`) trips a `debug_assert` in debug builds and is
        // clamped to an empty range at `start` in release builds, so the range
        // is never observably inverted; `len` therefore never underflows.
        assert_eq!(TextRange::new(4, 4), TextRange::empty(4));
        assert_eq!(TextRange::new(4, 4).len(), 0);
    }

    #[test]
    fn contains_offset() {
        let r = TextRange::new(2, 5);
        assert!(!r.contains(1));
        assert!(r.contains(2));
        assert!(r.contains(4));
        assert!(!r.contains(5));
        assert!(!TextRange::empty(3).contains(3));
    }

    #[test]
    fn intersects() {
        assert!(TextRange::new(0, 4).intersects(TextRange::new(2, 6)));
        assert!(!TextRange::new(0, 2).intersects(TextRange::new(2, 4)));
        assert!(!TextRange::new(0, 4).intersects(TextRange::empty(2)));
    }

    #[test]
    fn intersection() {
        assert_eq!(
            TextRange::new(0, 4).intersection(TextRange::new(2, 6)),
            Some(TextRange::new(2, 4))
        );
        assert_eq!(
            TextRange::new(0, 2).intersection(TextRange::new(5, 8)),
            None
        );
        assert_eq!(
            TextRange::new(0, 2).intersection(TextRange::new(2, 4)),
            None
        );
    }

    #[test]
    fn union() {
        assert_eq!(
            TextRange::new(2, 4).union(TextRange::new(8, 10)),
            TextRange::new(2, 10)
        );
        assert_eq!(
            TextRange::new(5, 9).union(TextRange::new(1, 6)),
            TextRange::new(1, 9)
        );
    }

    #[test]
    fn contains_range() {
        let outer = TextRange::new(2, 10);
        assert!(outer.contains_range(TextRange::new(4, 8)));
        assert!(outer.contains_range(TextRange::new(2, 10)));
        assert!(!outer.contains_range(TextRange::new(1, 5)));
        assert!(!outer.contains_range(TextRange::new(8, 12)));
        assert!(outer.contains_range(TextRange::empty(2)));
    }

    #[test]
    fn shifted_saturates() {
        assert_eq!(TextRange::new(3, 7).shifted(2), TextRange::new(5, 9));
        assert_eq!(TextRange::new(3, 7).shifted(-2), TextRange::new(1, 5));
        // Underflow saturates to zero.
        assert_eq!(TextRange::new(1, 3).shifted(-5), TextRange::empty(0));
        // Overflow saturates to usize::MAX.
        assert_eq!(
            TextRange::new(usize::MAX - 1, usize::MAX).shifted(5),
            TextRange::empty(usize::MAX)
        );
    }

    #[test]
    fn ordering_is_start_then_end() {
        assert!(TextRange::new(0, 5) < TextRange::new(1, 2));
        assert!(TextRange::new(0, 3) < TextRange::new(0, 4));
    }
}
