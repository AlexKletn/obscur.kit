//! `Hover`: the result of a `textDocument/hover` query — a label for the
//! innermost element under the cursor.

use crate::diagnostics::Range;

#[doc = include_str!("../../docs/api/analysis/hover.doc/Hover.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hover {
    /// The human-readable hover text.
    pub contents: Box<str>,
    /// The source range the hover describes.
    pub range: Range,
}

impl Hover {
    /// Creates a hover with `contents` over `range`.
    #[must_use]
    pub fn new(contents: impl Into<Box<str>>, range: Range) -> Self {
        Self {
            contents: contents.into(),
            range,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Position;

    #[test]
    fn stores_contents_and_range() {
        let range = Range::new(Position::new(0, 0), Position::new(0, 4));
        let hover = Hover::new("component Button", range);
        assert_eq!(&*hover.contents, "component Button");
        assert_eq!(hover.range, range);
    }
}
