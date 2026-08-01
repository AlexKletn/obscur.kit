#![doc = include_str!("text_span.doc/_module.md")]

use crate::span::{TextRange, TextStyle};

#[doc = include_str!("text_span.doc/TextSpan.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct TextSpan {
    #[doc = include_str!("text_span.doc/range.md")]
    pub range: TextRange,
    #[doc = include_str!("text_span.doc/style.md")]
    pub style: TextStyle,
}

impl TextSpan {
    #[doc = include_str!("text_span.doc/new.md")]
    #[must_use]
    pub const fn new(range: TextRange, style: TextStyle) -> Self {
        Self { range, style }
    }

    #[doc = include_str!("text_span.doc/range_2.md")]
    #[must_use]
    pub const fn range(&self) -> TextRange {
        self.range
    }

    #[doc = include_str!("text_span.doc/style_2.md")]
    #[must_use]
    pub const fn style(&self) -> &TextStyle {
        &self.style
    }

    #[doc = include_str!("text_span.doc/len.md")]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.range.len()
    }

    #[doc = include_str!("text_span.doc/is_empty.md")]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.range.is_empty()
    }

    #[doc = include_str!("text_span.doc/restyled.md")]
    #[must_use]
    pub fn restyled(&self, style: TextStyle) -> TextSpan {
        TextSpan {
            range: self.range,
            style,
        }
    }

    #[doc = include_str!("text_span.doc/with_range.md")]
    #[must_use]
    pub fn with_range(&self, range: TextRange) -> TextSpan {
        TextSpan {
            range,
            style: self.style.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::FontWeight;

    fn sample() -> TextSpan {
        TextSpan::new(TextRange::new(2, 7), TextStyle::new("Inter"))
    }

    #[test]
    fn accessors_and_len_delegation() {
        let span = sample();
        assert_eq!(span.range(), TextRange::new(2, 7));
        assert_eq!(span.style().font_family(), "Inter");
        assert_eq!(span.len(), 5);
        assert!(!span.is_empty());
        assert!(TextSpan::new(TextRange::empty(3), TextStyle::default()).is_empty());
    }

    #[test]
    fn restyled_keeps_range() {
        let span = sample();
        let bold = span.restyled(TextStyle::new("Inter").with_weight(FontWeight::BOLD));
        assert_eq!(bold.range(), span.range());
        assert_eq!(bold.style().weight(), FontWeight::BOLD);
        // Original is untouched.
        assert_eq!(span.style().weight(), FontWeight::NORMAL);
    }

    #[test]
    fn with_range_keeps_style() {
        let span = sample();
        let moved = span.with_range(TextRange::new(10, 20));
        assert_eq!(moved.range(), TextRange::new(10, 20));
        assert_eq!(moved.style(), span.style());
        assert_eq!(moved.len(), 10);
    }
}
