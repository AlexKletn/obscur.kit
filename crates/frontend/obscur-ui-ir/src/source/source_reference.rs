#![doc = include_str!("source_reference.doc/_module.md")]

use crate::source::SourceLocation;

#[doc = include_str!("source_reference.doc/SourceReference.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceReference {
    source: Box<str>,
    start: SourceLocation,
    end: SourceLocation,
}

impl SourceReference {
    #[doc = include_str!("source_reference.doc/new.md")]
    #[must_use]
    pub fn new(source: impl Into<Box<str>>, start: SourceLocation, end: SourceLocation) -> Self {
        Self {
            source: source.into(),
            start,
            end,
        }
    }

    #[doc = include_str!("source_reference.doc/point.md")]
    #[must_use]
    pub fn point(source: impl Into<Box<str>>, at: SourceLocation) -> Self {
        Self::new(source, at, at)
    }

    #[doc = include_str!("source_reference.doc/source.md")]
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    #[doc = include_str!("source_reference.doc/start.md")]
    #[must_use]
    pub const fn start(&self) -> SourceLocation {
        self.start
    }

    #[doc = include_str!("source_reference.doc/end.md")]
    #[must_use]
    pub const fn end(&self) -> SourceLocation {
        self.end
    }

    #[doc = include_str!("source_reference.doc/byte_len.md")]
    #[must_use]
    pub const fn byte_len(&self) -> u32 {
        self.end.offset().saturating_sub(self.start.offset())
    }

    #[doc = include_str!("source_reference.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_len_is_offset_difference() {
        let r = SourceReference::new(
            "app.okui",
            SourceLocation::new(4, 1, 5),
            SourceLocation::new(10, 1, 11),
        );
        assert_eq!(r.byte_len(), 6);
        assert_eq!(r.source(), "app.okui");
        assert!(!r.is_empty());
    }

    #[test]
    fn byte_len_saturates_on_inverted_offsets() {
        let r = SourceReference::new(
            "src",
            SourceLocation::new(10, 1, 1),
            SourceLocation::new(4, 1, 1),
        );
        assert_eq!(r.byte_len(), 0);
    }

    #[test]
    fn point_is_empty() {
        let r = SourceReference::point("src", SourceLocation::new(7, 2, 3));
        assert!(r.is_empty());
        assert_eq!(r.byte_len(), 0);
        assert_eq!(r.start(), r.end());
    }
}
