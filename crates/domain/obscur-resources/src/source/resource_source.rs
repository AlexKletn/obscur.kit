#![doc = include_str!("resource_source.doc/_module.md")]

use crate::source::{DocumentSource, EmbeddedSource, GeneratedSource, MemorySource};

#[doc = include_str!("resource_source.doc/ResourceSource.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceSource {
    #[doc = include_str!("resource_source.doc/Embedded.md")]
    Embedded(EmbeddedSource),
    #[doc = include_str!("resource_source.doc/Memory.md")]
    Memory(MemorySource),
    #[doc = include_str!("resource_source.doc/Document.md")]
    Document(DocumentSource),
    #[doc = include_str!("resource_source.doc/Generated.md")]
    Generated(GeneratedSource),
    #[doc = include_str!("resource_source.doc/File.md")]
    File(Box<str>),
    #[doc = include_str!("resource_source.doc/Url.md")]
    Url(Box<str>),
}

impl ResourceSource {
    #[doc = include_str!("resource_source.doc/embedded.md")]
    #[must_use]
    pub const fn embedded(source: EmbeddedSource) -> Self {
        Self::Embedded(source)
    }

    #[doc = include_str!("resource_source.doc/memory.md")]
    #[must_use]
    pub const fn memory(source: MemorySource) -> Self {
        Self::Memory(source)
    }

    #[doc = include_str!("resource_source.doc/document.md")]
    #[must_use]
    pub const fn document(source: DocumentSource) -> Self {
        Self::Document(source)
    }

    #[doc = include_str!("resource_source.doc/generated.md")]
    #[must_use]
    pub const fn generated(source: GeneratedSource) -> Self {
        Self::Generated(source)
    }

    #[doc = include_str!("resource_source.doc/file.md")]
    #[must_use]
    pub fn file(path: impl Into<Box<str>>) -> Self {
        Self::File(path.into())
    }

    #[doc = include_str!("resource_source.doc/url.md")]
    #[must_use]
    pub fn url(url: impl Into<Box<str>>) -> Self {
        Self::Url(url.into())
    }

    #[doc = include_str!("resource_source.doc/is_embedded.md")]
    #[must_use]
    pub const fn is_embedded(&self) -> bool {
        matches!(self, Self::Embedded(_))
    }

    #[doc = include_str!("resource_source.doc/is_memory.md")]
    #[must_use]
    pub const fn is_memory(&self) -> bool {
        matches!(self, Self::Memory(_))
    }

    #[doc = include_str!("resource_source.doc/is_document.md")]
    #[must_use]
    pub const fn is_document(&self) -> bool {
        matches!(self, Self::Document(_))
    }

    #[doc = include_str!("resource_source.doc/is_generated.md")]
    #[must_use]
    pub const fn is_generated(&self) -> bool {
        matches!(self, Self::Generated(_))
    }

    #[doc = include_str!("resource_source.doc/is_file.md")]
    #[must_use]
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    #[doc = include_str!("resource_source.doc/is_url.md")]
    #[must_use]
    pub const fn is_url(&self) -> bool {
        matches!(self, Self::Url(_))
    }

    #[doc = include_str!("resource_source.doc/requires_host_loader.md")]
    #[must_use]
    pub const fn requires_host_loader(&self) -> bool {
        matches!(self, Self::Url(_) | Self::Document(_))
    }

    #[doc = include_str!("resource_source.doc/inline_bytes.md")]
    #[must_use]
    pub fn inline_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Embedded(source) => Some(source.bytes()),
            Self::Memory(source) => Some(source.bytes()),
            Self::Document(_) | Self::Generated(_) | Self::File(_) | Self::Url(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_and_predicates_agree() {
        let embedded = ResourceSource::embedded(EmbeddedSource::new("a", vec![1u8]));
        let memory = ResourceSource::memory(MemorySource::new(vec![2u8]));
        let document = ResourceSource::document(DocumentSource::new("tok"));
        let generated = ResourceSource::generated(GeneratedSource::new("gen", "spec"));
        let file = ResourceSource::file("/path");
        let url = ResourceSource::url("https://example.com/a.png");

        assert!(embedded.is_embedded());
        assert!(memory.is_memory());
        assert!(document.is_document());
        assert!(generated.is_generated());
        assert!(file.is_file());
        assert!(url.is_url());

        assert!(!embedded.is_memory());
        assert!(!url.is_file());
    }

    #[test]
    fn requires_host_loader_only_for_url_and_document() {
        assert!(ResourceSource::url("https://x").requires_host_loader());
        assert!(ResourceSource::document(DocumentSource::new("t")).requires_host_loader());

        assert!(
            !ResourceSource::embedded(EmbeddedSource::new("a", vec![1u8])).requires_host_loader()
        );
        assert!(!ResourceSource::memory(MemorySource::new(vec![1u8])).requires_host_loader());
        assert!(!ResourceSource::generated(GeneratedSource::new("g", "s")).requires_host_loader());
        assert!(!ResourceSource::file("/p").requires_host_loader());
    }

    #[test]
    fn inline_bytes_some_for_embedded_and_memory() {
        let embedded = ResourceSource::embedded(EmbeddedSource::new("a", vec![1u8, 2]));
        let memory = ResourceSource::memory(MemorySource::new(vec![3u8, 4]));
        assert_eq!(embedded.inline_bytes(), Some(&[1u8, 2][..]));
        assert_eq!(memory.inline_bytes(), Some(&[3u8, 4][..]));
    }

    #[test]
    fn inline_bytes_none_for_deferred_kinds() {
        assert_eq!(
            ResourceSource::document(DocumentSource::new("t")).inline_bytes(),
            None
        );
        assert_eq!(
            ResourceSource::generated(GeneratedSource::new("g", "s")).inline_bytes(),
            None
        );
        assert_eq!(ResourceSource::file("/p").inline_bytes(), None);
        assert_eq!(ResourceSource::url("https://x").inline_bytes(), None);
    }

    #[test]
    fn equal_sources_are_equal() {
        let a = ResourceSource::memory(MemorySource::new(vec![1u8, 2, 3]));
        let b = ResourceSource::memory(MemorySource::new(vec![1u8, 2, 3]));
        assert_eq!(a, b);
    }
}
