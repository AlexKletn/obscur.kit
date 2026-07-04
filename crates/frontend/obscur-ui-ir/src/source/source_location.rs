//! A position within a source file [`SourceLocation`] (spec §2.5 source maps).

/// A single position within a source file (spec §2.5).
///
/// Records a byte `offset` together with the human-facing `line` and `column`
/// it corresponds to. Lines and columns are 1-based (the first character is at
/// line 1, column 1); the offset is 0-based. Ordering is by `offset` alone,
/// which is the canonical position within the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceLocation {
    offset: u32,
    line: u32,
    column: u32,
}

impl SourceLocation {
    /// Creates a location from a byte `offset` (0-based) and its `line` and
    /// `column` (both 1-based).
    #[must_use]
    pub const fn new(offset: u32, line: u32, column: u32) -> Self {
        Self {
            offset,
            line,
            column,
        }
    }

    /// Returns the location at the very start of a source: offset 0, line 1,
    /// column 1.
    #[must_use]
    pub const fn start() -> Self {
        Self::new(0, 1, 1)
    }

    /// Returns the 0-based byte offset.
    #[must_use]
    pub const fn offset(self) -> u32 {
        self.offset
    }

    /// Returns the 1-based line number.
    #[must_use]
    pub const fn line(self) -> u32 {
        self.line
    }

    /// Returns the 1-based column number.
    #[must_use]
    pub const fn column(self) -> u32 {
        self.column
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_is_origin() {
        let s = SourceLocation::start();
        assert_eq!(s.offset(), 0);
        assert_eq!(s.line(), 1);
        assert_eq!(s.column(), 1);
    }

    #[test]
    fn accessors_return_components() {
        let loc = SourceLocation::new(42, 3, 7);
        assert_eq!(loc.offset(), 42);
        assert_eq!(loc.line(), 3);
        assert_eq!(loc.column(), 7);
    }

    #[test]
    fn ordering_is_by_offset() {
        let a = SourceLocation::new(10, 5, 1);
        let b = SourceLocation::new(20, 1, 1);
        assert!(a < b);
        // Ordering compares offset first, so a smaller offset wins regardless
        // of a larger line/column.
        assert!(SourceLocation::new(10, 99, 99) < SourceLocation::new(11, 1, 1));
    }
}
