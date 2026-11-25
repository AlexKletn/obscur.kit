//! `Position` and its paired [`Range`]: zero-based text coordinates used by the
//! language-server core.

#[doc = include_str!("../../docs/api/diagnostics/position.doc/Position.md")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_orders_by_line_then_character() {
        assert!(Position::new(0, 5) < Position::new(1, 0));
        assert!(Position::new(1, 1) > Position::new(1, 0));
    }
}
