#![doc = include_str!("hover.doc/_module.md")]

use crate::diagnostics::Range;

#[doc = include_str!("hover.doc/Hover.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hover {
    #[doc = include_str!("hover.doc/contents.md")]
    pub contents: Box<str>,
    #[doc = include_str!("hover.doc/range.md")]
    pub range: Range,
}

impl Hover {
    #[doc = include_str!("hover.doc/new.md")]
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
