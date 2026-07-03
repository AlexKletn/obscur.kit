//! A runtime in-memory resource source (spec §5.15 memory, §5.16 bytes).

use std::sync::Arc;

/// A resource backed by bytes produced at runtime (spec §5.15 memory source,
/// §5.16 image bytes).
///
/// Unlike [`crate::source::EmbeddedSource`], whose bytes are baked into the binary
/// at build time, a memory source holds bytes that were produced at runtime — for
/// example decoded, downloaded by the host, or generated in process. It optionally
/// carries a MIME content-type hint the loader may use when interpreting them.
///
/// The bytes are held in an [`Arc<[u8]>`] so clones are cheap and two sources are
/// compared and hashed by their byte *content*, which lets the hub deduplicate
/// equal sources (spec §5.15).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemorySource {
    bytes: Arc<[u8]>,
    mime_type: Option<Box<str>>,
}

impl MemorySource {
    /// Creates a memory source from runtime bytes, with no MIME hint.
    #[must_use]
    pub fn new(bytes: impl Into<Arc<[u8]>>) -> Self {
        Self {
            bytes: bytes.into(),
            mime_type: None,
        }
    }

    /// Returns this source with its MIME content-type hint set (spec §5.16).
    #[must_use]
    pub fn with_mime_type(mut self, mime_type: impl Into<Box<str>>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    /// Returns the runtime bytes (spec §5.16).
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the MIME content-type hint, if any (spec §5.16).
    #[must_use]
    pub fn mime_type(&self) -> Option<&str> {
        self.mime_type.as_deref()
    }

    /// Returns the number of bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns whether there are no bytes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn accessors_and_len() {
        let src = MemorySource::new(vec![4u8, 5, 6]);
        assert_eq!(src.bytes(), &[4, 5, 6]);
        assert_eq!(src.len(), 3);
        assert!(!src.is_empty());
        assert_eq!(src.mime_type(), None);
    }

    #[test]
    fn empty_reports_empty() {
        let src = MemorySource::new(Vec::<u8>::new());
        assert!(src.is_empty());
    }

    #[test]
    fn with_mime_type_sets_hint() {
        let src = MemorySource::new(vec![1u8]).with_mime_type("image/png");
        assert_eq!(src.mime_type(), Some("image/png"));
    }

    #[test]
    fn equal_content_dedups_in_hash_set() {
        let a = MemorySource::new(vec![7u8, 7, 7]);
        let b = MemorySource::new(vec![7u8, 7, 7]);
        assert_eq!(a, b);
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn mime_hint_participates_in_equality() {
        let a = MemorySource::new(vec![1u8]);
        let b = MemorySource::new(vec![1u8]).with_mime_type("image/png");
        assert_ne!(a, b);
    }
}
