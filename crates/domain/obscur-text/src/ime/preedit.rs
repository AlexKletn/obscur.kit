//! The uncommitted composition string shown inline during IME input, kept
//! separate from committed text (spec §5.12 IME).

use crate::span::TextRange;

#[doc = include_str!("../../docs/api/ime/preedit.doc/Preedit.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preedit {
    /// The uncommitted composition text.
    text: Box<str>,
    /// Caret byte offset within `text`; clamped to `text.len()` and snapped to a
    /// char boundary.
    caret: usize,
    /// The emphasized active/target clause, as a byte range within `text`.
    highlight: Option<TextRange>,
}

impl Preedit {
    /// Creates a preedit holding `text`, with the caret at the end of the text
    /// and no highlight.
    #[must_use]
    pub fn new(text: impl Into<Box<str>>) -> Self {
        let text = text.into();
        let caret = text.len();
        Self {
            text,
            caret,
            highlight: None,
        }
    }

    /// Creates an empty preedit (empty text, caret at `0`, no highlight).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            text: Box::from(""),
            caret: 0,
            highlight: None,
        }
    }

    /// Returns a copy with the caret set to `caret`, clamped to the text length
    /// and snapped down to the nearest UTF-8 character boundary.
    #[must_use]
    pub fn with_caret(mut self, caret: usize) -> Self {
        self.caret = snap_caret(&self.text, caret);
        self
    }

    #[doc = include_str!("../../docs/api/ime/preedit.doc/with_highlight.md")]
    #[must_use]
    pub fn with_highlight(mut self, highlight: TextRange) -> Self {
        self.highlight = Some(highlight);
        self
    }

    /// Returns the uncommitted composition text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the caret byte offset within the preedit text.
    #[must_use]
    pub const fn caret(&self) -> usize {
        self.caret
    }

    /// Returns the emphasized clause range within the preedit text, if any.
    #[must_use]
    pub const fn highlight(&self) -> Option<TextRange> {
        self.highlight
    }

    /// Returns whether the preedit text is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Returns the byte length of the preedit text.
    #[must_use]
    pub fn len(&self) -> usize {
        self.text.len()
    }
}

/// Clamps `caret` to `text.len()` and snaps it down to the nearest UTF-8
/// character boundary, so the returned offset always indexes `text` safely.
fn snap_caret(text: &str, caret: usize) -> usize {
    let mut caret = caret.min(text.len());
    while !text.is_char_boundary(caret) {
        caret -= 1;
    }
    caret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_puts_caret_at_end_without_highlight() {
        let p = Preedit::new("abc");
        assert_eq!(p.text(), "abc");
        assert_eq!(p.caret(), 3);
        assert_eq!(p.highlight(), None);
        assert!(!p.is_empty());
        assert_eq!(p.len(), 3);
    }

    #[test]
    fn empty_is_empty() {
        let p = Preedit::empty();
        assert!(p.is_empty());
        assert_eq!(p.len(), 0);
        assert_eq!(p.caret(), 0);
        assert_eq!(p.text(), "");
    }

    #[test]
    fn caret_clamped_to_len() {
        let p = Preedit::new("ab").with_caret(99);
        assert_eq!(p.caret(), 2);
    }

    #[test]
    fn caret_snapped_to_char_boundary() {
        // "é" is two bytes (0xC3 0xA9); offset 1 is inside it and snaps to 0.
        let p = Preedit::new("é").with_caret(1);
        assert_eq!(p.len(), 2);
        assert_eq!(p.caret(), 0);
        // A valid boundary is kept.
        let p = Preedit::new("aé").with_caret(1);
        assert_eq!(p.caret(), 1);
    }

    #[test]
    fn with_highlight_sets_range() {
        let p = Preedit::new("hello").with_highlight(TextRange::new(0, 2));
        assert_eq!(p.highlight(), Some(TextRange::new(0, 2)));
    }
}
