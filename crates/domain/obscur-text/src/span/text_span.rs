//! A styled run of text: a byte range paired with a typography style (spec §5.10).

use crate::span::{TextRange, TextStyle};

/// A styled run of text (spec §5.10): a [`TextRange`] paired with the
/// [`TextStyle`] applied to those bytes.
///
/// Rich text is described by a sequence of non-overlapping spans; managing that
/// sequence (ordering, coalescing, splitting) is the job of the buffer and
/// layout modules built later. A single `TextSpan` models just one contiguous
/// styled range.
///
/// Because [`TextStyle`] holds `f32` fields, `TextSpan` derives only `Debug`,
/// `Clone`, and `PartialEq` (floats preclude `Eq`).
#[derive(Debug, Clone, PartialEq)]
pub struct TextSpan {
    /// The byte range these bytes cover.
    pub range: TextRange,
    /// The typography applied to the range.
    pub style: TextStyle,
}

impl TextSpan {
    /// Creates a span applying `style` to `range`.
    #[must_use]
    pub const fn new(range: TextRange, style: TextStyle) -> Self {
        Self { range, style }
    }

    /// Returns the byte range this span covers.
    #[must_use]
    pub const fn range(&self) -> TextRange {
        self.range
    }

    /// Returns the typography applied to the range.
    #[must_use]
    pub const fn style(&self) -> &TextStyle {
        &self.style
    }

    /// Returns the length of the span in bytes, delegating to the range.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.range.len()
    }

    /// Returns whether the span covers no bytes, delegating to the range.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.range.is_empty()
    }

    /// Returns a copy of this span with a new style applied to the same range.
    #[must_use]
    pub fn restyled(&self, style: TextStyle) -> TextSpan {
        TextSpan {
            range: self.range,
            style,
        }
    }

    /// Returns a copy of this span moved to a new range, keeping the style.
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
