#![doc = include_str!("embedded_source.doc/_module.md")]

use std::sync::Arc;

#[doc = include_str!("embedded_source.doc/EmbeddedSource.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmbeddedSource {
    name: Box<str>,
    bytes: Arc<[u8]>,
}

impl EmbeddedSource {
    #[doc = include_str!("embedded_source.doc/new.md")]
    #[must_use]
    pub fn new(name: impl Into<Box<str>>, bytes: impl Into<Arc<[u8]>>) -> Self {
        Self {
            name: name.into(),
            bytes: bytes.into(),
        }
    }

    #[doc = include_str!("embedded_source.doc/name.md")]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[doc = include_str!("embedded_source.doc/bytes.md")]
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[doc = include_str!("embedded_source.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    #[doc = include_str!("embedded_source.doc/is_empty.md")]
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
    fn accessors_expose_name_and_bytes() {
        let src = EmbeddedSource::new("logo", vec![1u8, 2, 3]);
        assert_eq!(src.name(), "logo");
        assert_eq!(src.bytes(), &[1, 2, 3]);
        assert_eq!(src.len(), 3);
        assert!(!src.is_empty());
    }

    #[test]
    fn empty_bytes_report_empty() {
        let src = EmbeddedSource::new("blank", Vec::<u8>::new());
        assert_eq!(src.len(), 0);
        assert!(src.is_empty());
    }

    #[test]
    fn equal_content_dedups_in_hash_set() {
        let a = EmbeddedSource::new("logo", vec![9u8, 8, 7]);
        let b = EmbeddedSource::new("logo", vec![9u8, 8, 7]);
        assert_eq!(a, b);
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn different_name_or_bytes_are_distinct() {
        let a = EmbeddedSource::new("logo", vec![1u8]);
        let b = EmbeddedSource::new("icon", vec![1u8]);
        let c = EmbeddedSource::new("logo", vec![2u8]);
        assert_ne!(a, b);
        assert_ne!(a, c);
    }
}
