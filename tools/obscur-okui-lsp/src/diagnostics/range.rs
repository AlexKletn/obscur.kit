use super::Position;

/// A half-open span between two [`Position`]s (`start` inclusive, `end`
/// exclusive), mirroring the LSP `Range` shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Range {
    /// The start of the span (inclusive).
    pub start: Position,
    /// The end of the span (exclusive).
    pub end: Position,
}

impl Range {
    /// Creates a range from `start` to `end`.
    #[must_use]
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Creates an empty (zero-width) range anchored at `position`.
    #[must_use]
    pub const fn empty(position: Position) -> Self {
        Self {
            start: position,
            end: position,
        }
    }

    /// Returns `true` if `position` lies within this range (`start <= position <
    /// end`), or equals `start` for an empty range.
    #[must_use]
    pub fn contains(&self, position: Position) -> bool {
        if self.start == self.end {
            position == self.start
        } else {
            position >= self.start && position < self.end
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_contains_is_half_open() {
        let range = Range::new(Position::new(0, 2), Position::new(0, 5));
        assert!(!range.contains(Position::new(0, 1)));
        assert!(range.contains(Position::new(0, 2)));
        assert!(range.contains(Position::new(0, 4)));
        assert!(!range.contains(Position::new(0, 5)));
    }

    #[test]
    fn empty_range_contains_only_its_anchor() {
        let range = Range::empty(Position::new(3, 7));
        assert!(range.contains(Position::new(3, 7)));
        assert!(!range.contains(Position::new(3, 8)));
    }
}
