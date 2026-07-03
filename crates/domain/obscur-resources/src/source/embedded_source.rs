//! A compile-time-baked resource source (spec §5.15 embedded).

use std::sync::Arc;

/// A resource whose bytes are baked into the binary at build time (spec §5.15
/// embedded source).
///
/// An embedded source names a logical asset and carries the immutable bytes that
/// were compiled into the program (for example via `include_bytes!`). It is a
/// pure description: the bytes are already in hand, so no loader, host, or I/O is
/// needed to resolve them.
///
/// The bytes are held in an [`Arc<[u8]>`] so clones are cheap and two sources are
/// compared and hashed by their byte *content*, which lets the hub deduplicate
/// equal sources (spec §5.15). This differs from [`crate::source::MemorySource`],
/// whose bytes are produced at runtime rather than baked at build time.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmbeddedSource {
    name: Box<str>,
    bytes: Arc<[u8]>,
}

impl EmbeddedSource {
    /// Creates an embedded source from a logical asset name and its baked bytes.
    #[must_use]
    pub fn new(name: impl Into<Box<str>>, bytes: impl Into<Arc<[u8]>>) -> Self {
        Self {
            name: name.into(),
            bytes: bytes.into(),
        }
    }

    /// Returns the logical asset name (spec §5.15).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the baked bytes (spec §5.15).
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the number of baked bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns whether there are no baked bytes.
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
