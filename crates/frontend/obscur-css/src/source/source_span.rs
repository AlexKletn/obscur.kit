//! A byte range within a CSS source (spec §2.5 source maps).

#[doc = include_str!("../../docs/api/source/source_span.doc/SourceSpan.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    start: u32,
    end: u32,
}

impl SourceSpan {
    #[doc = include_str!("../../docs/api/source/source_span.doc/new.md")]
    #[must_use]
    pub const fn new(start: u32, end: u32) -> Self {
        let end = if end < start { start } else { end };
        Self { start, end }
    }

    /// Creates an empty span (`start == end`) at the byte offset `at`.
    #[must_use]
    pub const fn empty(at: u32) -> Self {
        Self { start: at, end: at }
    }

    /// Creates an empty span at the byte offset `at`; an alias for
    /// [`SourceSpan::empty`].
    #[must_use]
    pub const fn point(at: u32) -> Self {
        Self::empty(at)
    }

    /// Returns the start byte offset of the span.
    #[must_use]
    pub const fn start(&self) -> u32 {
        self.start
    }

    /// Returns the end byte offset of the span (exclusive).
    #[must_use]
    pub const fn end(&self) -> u32 {
        self.end
    }

    /// Returns the length of the span in bytes (`end - start`), saturating to `0`.
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    /// Returns whether the span is empty (`start == end`).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Returns whether `offset` lies within the span (`start <= offset < end`).
    #[must_use]
    pub const fn contains(&self, offset: u32) -> bool {
        self.start <= offset && offset < self.end
    }

    /// Returns the smallest span covering both `self` and `other` (min start,
    /// max end).
    #[must_use]
    pub const fn union(&self, other: SourceSpan) -> SourceSpan {
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
        Self { start, end }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn len_is_byte_width() {
        let span = SourceSpan::new(4, 10);
        assert_eq!(span.start(), 4);
        assert_eq!(span.end(), 10);
        assert_eq!(span.len(), 6);
        assert!(!span.is_empty());
    }

    #[test]
    fn empty_and_point_are_zero_length() {
        let e = SourceSpan::empty(5);
        let p = SourceSpan::point(5);
        assert_eq!(e, p);
        assert!(e.is_empty());
        assert_eq!(e.len(), 0);
        assert_eq!(e.start(), 5);
        assert_eq!(e.end(), 5);
    }

    #[test]
    fn inverted_range_clamps_to_empty() {
        let span = SourceSpan::new(10, 4);
        assert_eq!(span.start(), 10);
        assert_eq!(span.end(), 10);
        assert_eq!(span.len(), 0);
        assert!(span.is_empty());
    }

    #[test]
    fn contains_is_half_open() {
        let span = SourceSpan::new(3, 6);
        assert!(!span.contains(2));
        assert!(span.contains(3));
        assert!(span.contains(5));
        assert!(!span.contains(6));
    }

    #[test]
    fn union_spans_both() {
        let a = SourceSpan::new(2, 5);
        let b = SourceSpan::new(8, 12);
        let u = a.union(b);
        assert_eq!(u.start(), 2);
        assert_eq!(u.end(), 12);
        assert_eq!(u, b.union(a));
    }
}
