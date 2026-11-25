//! An owned CSS source: its id, logical name, and text (spec §2.5 source maps).

use crate::source::{SourceId, SourceSpan, SourceVersion};

#[doc = include_str!("../../docs/api/source/css_source.doc/CssSource.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssSource {
    id: SourceId,
    /// Which revision of this source's text `text` is (spec §4.12). Stamped by
    /// the owner; a cache compares it rather than the text.
    version: SourceVersion,
    name: Box<str>,
    text: Box<str>,
}

impl CssSource {
    /// Creates a source with the given `id`, logical `name`, and `text`.
    #[must_use]
    pub fn new(id: SourceId, name: impl Into<Box<str>>, text: impl Into<Box<str>>) -> Self {
        Self {
            id,
            version: SourceVersion::INITIAL,
            name: name.into(),
            text: text.into(),
        }
    }

    /// Creates an anonymous source ([`SourceId::ANONYMOUS`], empty name) from
    /// `text`.
    #[must_use]
    pub fn anonymous(text: impl Into<Box<str>>) -> Self {
        Self::new(SourceId::ANONYMOUS, "", text)
    }

    #[doc = include_str!("../../docs/api/source/css_source.doc/at_version.md")]
    #[must_use]
    pub fn at_version(mut self, version: SourceVersion) -> Self {
        self.version = version;
        self
    }

    /// The revision of this source's text.
    #[must_use]
    pub const fn version(&self) -> SourceVersion {
        self.version
    }

    /// Returns the id of this source.
    #[must_use]
    pub const fn id(&self) -> SourceId {
        self.id
    }

    /// Returns the logical name of this source.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the full source text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the length of the source text in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Returns whether the source text is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    #[doc = include_str!("../../docs/api/source/css_source.doc/slice.md")]
    #[must_use]
    pub fn slice(&self, span: SourceSpan) -> &str {
        let text = &self.text;
        let max = text.len();
        // `u32 as usize` is a widening cast on every supported target, so it is
        // lossless; use `usize::try_from` to stay away from `as`.
        let clamp = |offset: u32| -> usize {
            let mut idx = usize::try_from(offset).unwrap_or(max).min(max);
            while idx > 0 && !text.is_char_boundary(idx) {
                idx -= 1;
            }
            idx
        };
        let start = clamp(span.start());
        let end = clamp(span.end());
        if start >= end {
            return "";
        }
        &text[start..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_report_fields() {
        let src = CssSource::new(SourceId::from_raw(3), "theme.css", "a { color: red }");
        assert_eq!(src.id(), SourceId::from_raw(3));
        assert_eq!(src.name(), "theme.css");
        assert_eq!(src.text(), "a { color: red }");
        assert_eq!(src.len(), 16);
        assert!(!src.is_empty());
    }

    #[test]
    fn anonymous_uses_reserved_id_and_empty_name() {
        let src = CssSource::anonymous("body {}");
        assert_eq!(src.id(), SourceId::ANONYMOUS);
        assert_eq!(src.name(), "");
        assert_eq!(src.text(), "body {}");
    }

    #[test]
    fn slice_in_bounds() {
        let src = CssSource::anonymous("hello world");
        assert_eq!(src.slice(SourceSpan::new(0, 5)), "hello");
        assert_eq!(src.slice(SourceSpan::new(6, 11)), "world");
    }

    #[test]
    fn slice_out_of_bounds_clamps() {
        let src = CssSource::anonymous("abc");
        assert_eq!(src.slice(SourceSpan::new(1, 999)), "bc");
        assert_eq!(src.slice(SourceSpan::new(500, 900)), "");
        assert_eq!(src.slice(SourceSpan::empty(2)), "");
    }

    #[test]
    fn slice_multibyte_never_panics() {
        // "é" is 2 bytes (0..2), "😀" is 4 bytes (2..6), "z" is 1 byte (6..7);
        // char boundaries are at 0, 2, 6, 7.
        let text = "é😀z";
        let src = CssSource::anonymous(text);
        // Span exactly on boundaries returns the emoji.
        assert_eq!(src.slice(SourceSpan::new(2, 6)), "😀");
        // A span cutting through characters snaps its ends down to boundaries:
        // start 1 -> 0, end 4 -> 2, yielding "é".
        assert_eq!(src.slice(SourceSpan::new(1, 4)), "é");
        // The whole text round-trips.
        let full = SourceSpan::new(0, u32::try_from(text.len()).unwrap());
        assert_eq!(src.slice(full), text);
        // A span whose ends snap to the same boundary collapses safely.
        assert_eq!(src.slice(SourceSpan::new(3, 5)), "");
    }
}
