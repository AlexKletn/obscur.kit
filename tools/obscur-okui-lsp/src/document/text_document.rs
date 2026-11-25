//! `TextDocument`: an in-memory open document, and [`LineIndex`], its
//! precomputed line-offset table for byte-offset ↔ [`Position`] mapping.

#[cfg(test)]
use crate::diagnostics::Position;

use super::LineIndex;

#[doc = include_str!("../../docs/api/document/text_document.doc/TextDocument.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextDocument {
    /// The document's URI (an opaque client-supplied identifier).
    pub uri: Box<str>,
    /// The document version, as supplied by the client.
    pub version: i32,
    /// The full document text.
    pub text: String,
}

impl TextDocument {
    /// Creates a document for `uri` at `version` holding `text`.
    #[must_use]
    pub fn new(uri: impl Into<Box<str>>, version: i32, text: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            version,
            text: text.into(),
        }
    }

    /// Builds a fresh [`LineIndex`] over this document's current text.
    #[must_use]
    pub fn line_index(&self) -> LineIndex {
        LineIndex::new(&self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_document_builds_its_own_index() {
        let doc = TextDocument::new("file:///a.okui", 1, "<A/>\n<B/>");
        let index = doc.line_index();
        assert_eq!(index.offset_to_position(5), Position::new(1, 0));
        assert_eq!(doc.version, 1);
    }
}
