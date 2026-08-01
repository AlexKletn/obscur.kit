#![doc = include_str!("position.doc/_module.md")]

#[doc = include_str!("position.doc/Position.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Position {
    #[doc = include_str!("position.doc/line.md")]
    pub line: u32,
    #[doc = include_str!("position.doc/character.md")]
    pub character: u32,
}

impl Position {
    #[doc = include_str!("position.doc/new.md")]
    #[must_use]
    pub const fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

#[doc = include_str!("position.doc/Range.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Range {
    #[doc = include_str!("position.doc/start.md")]
    pub start: Position,
    #[doc = include_str!("position.doc/end.md")]
    pub end: Position,
}

impl Range {
    #[doc = include_str!("position.doc/new_2.md")]
    #[must_use]
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    #[doc = include_str!("position.doc/empty.md")]
    #[must_use]
    pub const fn empty(position: Position) -> Self {
        Self {
            start: position,
            end: position,
        }
    }

    #[doc = include_str!("position.doc/contains.md")]
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
