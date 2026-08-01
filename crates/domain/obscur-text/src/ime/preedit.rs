#![doc = include_str!("preedit.doc/_module.md")]

use crate::span::TextRange;

#[doc = include_str!("preedit.doc/Preedit.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preedit {
    #[doc = include_str!("preedit.doc/text.md")]
    text: Box<str>,
    #[doc = include_str!("preedit.doc/caret.md")]
    caret: usize,
    #[doc = include_str!("preedit.doc/highlight.md")]
    highlight: Option<TextRange>,
}

impl Preedit {
    #[doc = include_str!("preedit.doc/new.md")]
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

    #[doc = include_str!("preedit.doc/empty.md")]
    #[must_use]
    pub fn empty() -> Self {
        Self {
            text: Box::from(""),
            caret: 0,
            highlight: None,
        }
    }

    #[doc = include_str!("preedit.doc/with_caret.md")]
    #[must_use]
    pub fn with_caret(mut self, caret: usize) -> Self {
        self.caret = snap_caret(&self.text, caret);
        self
    }

    #[doc = include_str!("preedit.doc/with_highlight.md")]
    #[must_use]
    pub fn with_highlight(mut self, highlight: TextRange) -> Self {
        self.highlight = Some(highlight);
        self
    }

    #[doc = include_str!("preedit.doc/text_2.md")]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[doc = include_str!("preedit.doc/caret_2.md")]
    #[must_use]
    pub const fn caret(&self) -> usize {
        self.caret
    }

    #[doc = include_str!("preedit.doc/highlight_2.md")]
    #[must_use]
    pub const fn highlight(&self) -> Option<TextRange> {
        self.highlight
    }

    #[doc = include_str!("preedit.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    #[doc = include_str!("preedit.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.text.len()
    }
}

#[doc = include_str!("preedit.doc/snap_caret.md")]
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
