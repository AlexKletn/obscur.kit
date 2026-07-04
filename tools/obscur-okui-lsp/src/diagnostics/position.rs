//! `Position` and its paired [`Range`]: zero-based text coordinates used by the
//! language-server core.

/// A zero-based position in a text document.
///
/// `line` is the zero-based line number (newlines split lines) and `character`
/// is the zero-based **UTF-8 byte** offset from the start of that line.
///
/// # Column unit
///
/// The Language Server Protocol defines `character` as a UTF-16 code-unit
/// offset. This transport-agnostic core instead uses a **UTF-8 byte** column so
/// coordinates map losslessly to and from the byte offsets carried by the okui
/// parser and compiler diagnostics (which are byte offsets). A JSON-RPC wrapper
/// that must speak strict LSP can re-encode the column against the line's text;
/// for a byte-oriented `.okui` checker the byte column is exact and stable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Position {
    /// Zero-based line number.
    pub line: u32,
    /// Zero-based UTF-8 byte offset within the line.
    pub character: u32,
}

impl Position {
    /// Creates a position at `line` and `character` (both zero-based).
    #[must_use]
    pub const fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

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
    fn position_orders_by_line_then_character() {
        assert!(Position::new(0, 5) < Position::new(1, 0));
        assert!(Position::new(1, 1) > Position::new(1, 0));
    }

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
