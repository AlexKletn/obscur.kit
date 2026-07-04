//! `DocumentStore`: the set of documents currently open in the language server.

use std::collections::HashMap;

use crate::document::TextDocument;

/// An in-memory map from document URI to its open [`TextDocument`].
///
/// The store mirrors the LSP document-lifecycle notifications: `didOpen` →
/// [`open`](Self::open), `didChange` → [`change`](Self::change) (full-text
/// synchronization) and `didClose` → [`close`](Self::close). It is the single
/// source of truth the [`Analyzer`](crate::Analyzer) reads from.
#[derive(Debug, Clone, Default)]
pub struct DocumentStore {
    documents: HashMap<Box<str>, TextDocument>,
}

impl DocumentStore {
    /// Creates an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens (or replaces) the document at `uri` with `version` and `text`,
    /// returning a reference to the stored document.
    pub fn open(&mut self, uri: impl Into<Box<str>>, version: i32, text: impl Into<String>) {
        let uri: Box<str> = uri.into();
        self.documents
            .insert(uri.clone(), TextDocument::new(uri, version, text));
    }

    /// Applies a full-text change to an already-open document: bumps its version
    /// and replaces its text. Returns `true` if the document existed.
    pub fn change(&mut self, uri: &str, version: i32, new_text: impl Into<String>) -> bool {
        if let Some(document) = self.documents.get_mut(uri) {
            document.version = version;
            document.text = new_text.into();
            true
        } else {
            false
        }
    }

    /// Closes the document at `uri`, returning it if it was open.
    pub fn close(&mut self, uri: &str) -> Option<TextDocument> {
        self.documents.remove(uri)
    }

    /// Returns the open document at `uri`, if any.
    #[must_use]
    pub fn get(&self, uri: &str) -> Option<&TextDocument> {
        self.documents.get(uri)
    }

    /// Returns the number of open documents.
    #[must_use]
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Returns whether the store holds no documents.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_get_len() {
        let mut store = DocumentStore::new();
        assert!(store.is_empty());
        store.open("file:///a.okui", 1, "<A/>");
        assert_eq!(store.len(), 1);
        let doc = store.get("file:///a.okui").expect("open document");
        assert_eq!(doc.version, 1);
        assert_eq!(doc.text, "<A/>");
    }

    #[test]
    fn change_bumps_version_and_replaces_text() {
        let mut store = DocumentStore::new();
        store.open("file:///a.okui", 1, "<A/>");
        assert!(store.change("file:///a.okui", 2, "<B/>"));
        let doc = store.get("file:///a.okui").expect("open document");
        assert_eq!(doc.version, 2);
        assert_eq!(doc.text, "<B/>");
    }

    #[test]
    fn change_on_missing_document_reports_false() {
        let mut store = DocumentStore::new();
        assert!(!store.change("file:///missing.okui", 2, "x"));
    }

    #[test]
    fn close_removes_and_returns_the_document() {
        let mut store = DocumentStore::new();
        store.open("file:///a.okui", 1, "<A/>");
        let closed = store.close("file:///a.okui").expect("a closed document");
        assert_eq!(closed.text, "<A/>");
        assert!(store.get("file:///a.okui").is_none());
        assert!(store.close("file:///a.okui").is_none());
    }

    #[test]
    fn open_replaces_an_existing_document() {
        let mut store = DocumentStore::new();
        store.open("file:///a.okui", 1, "<A/>");
        store.open("file:///a.okui", 5, "<C/>");
        assert_eq!(store.len(), 1);
        let doc = store.get("file:///a.okui").expect("open document");
        assert_eq!(doc.version, 5);
        assert_eq!(doc.text, "<C/>");
    }
}
