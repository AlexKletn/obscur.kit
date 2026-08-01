#![doc = include_str!("memory_source.doc/_module.md")]

use std::sync::Arc;

#[doc = include_str!("memory_source.doc/MemorySource.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemorySource {
    bytes: Arc<[u8]>,
    mime_type: Option<Box<str>>,
}

impl MemorySource {
    #[doc = include_str!("memory_source.doc/new.md")]
    #[must_use]
    pub fn new(bytes: impl Into<Arc<[u8]>>) -> Self {
        Self {
            bytes: bytes.into(),
            mime_type: None,
        }
    }

    #[doc = include_str!("memory_source.doc/with_mime_type.md")]
    #[must_use]
    pub fn with_mime_type(mut self, mime_type: impl Into<Box<str>>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    #[doc = include_str!("memory_source.doc/bytes.md")]
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[doc = include_str!("memory_source.doc/mime_type.md")]
    #[must_use]
    pub fn mime_type(&self) -> Option<&str> {
        self.mime_type.as_deref()
    }

    #[doc = include_str!("memory_source.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    #[doc = include_str!("memory_source.doc/is_empty.md")]
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
