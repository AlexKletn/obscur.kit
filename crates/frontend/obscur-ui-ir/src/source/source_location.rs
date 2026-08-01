#![doc = include_str!("source_location.doc/_module.md")]

#[doc = include_str!("source_location.doc/SourceLocation.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceLocation {
    offset: u32,
    line: u32,
    column: u32,
}

impl SourceLocation {
    #[doc = include_str!("source_location.doc/new.md")]
    #[must_use]
    pub const fn new(offset: u32, line: u32, column: u32) -> Self {
        Self {
            offset,
            line,
            column,
        }
    }

    #[doc = include_str!("source_location.doc/start.md")]
    #[must_use]
    pub const fn start() -> Self {
        Self::new(0, 1, 1)
    }

    #[doc = include_str!("source_location.doc/offset.md")]
    #[must_use]
    pub const fn offset(self) -> u32 {
        self.offset
    }

    #[doc = include_str!("source_location.doc/line.md")]
    #[must_use]
    pub const fn line(self) -> u32 {
        self.line
    }

    #[doc = include_str!("source_location.doc/column.md")]
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
