//! The umbrella enum of every resource source kind (spec §5.15).

use crate::source::{DocumentSource, EmbeddedSource, GeneratedSource, MemorySource};

/// Where a resource's bytes come from (spec §5.15 sources).
///
/// This is a pure *description* of a resource's origin that a loader port resolves
/// later; it performs no I/O itself. Per spec §5.15 the recognised source kinds
/// are: embedded (baked at build), file, memory, URL (through a user/host loader),
/// generated content, plus a host document handle for picked files (spec §5.16).
///
/// The bytes-in-hand kinds ([`EmbeddedSource`], [`MemorySource`]) can be resolved
/// entirely within the kit; the remaining kinds — file, URL, and document — are
/// resolved outside the kit by a loader or host. In particular the kit performs no
/// arbitrary network access by itself (spec §5.15): a [`ResourceSource::Url`] is
/// only a request that a user/host loader chooses to fulfil.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceSource {
    /// A compile-time-baked asset (spec §5.15 embedded).
    Embedded(EmbeddedSource),
    /// Runtime in-memory bytes (spec §5.15 memory).
    Memory(MemorySource),
    /// A host document handle, for example a picked file (spec §5.16, §6.10).
    Document(DocumentSource),
    /// Procedurally generated content (spec §5.16).
    Generated(GeneratedSource),
    /// A path resolved by the loader port (spec §5.15 file). The payload is an
    /// opaque path string the host interprets; the kit does not open it here.
    File(Box<str>),
    /// A URL resolved *through* a user/host loader (spec §5.15). The kit performs
    /// no network access by itself; the payload is only the requested URL string.
    Url(Box<str>),
}

impl ResourceSource {
    /// Wraps an [`EmbeddedSource`] (spec §5.15 embedded).
    #[must_use]
    pub const fn embedded(source: EmbeddedSource) -> Self {
        Self::Embedded(source)
    }

    /// Wraps a [`MemorySource`] (spec §5.15 memory).
    #[must_use]
    pub const fn memory(source: MemorySource) -> Self {
        Self::Memory(source)
    }

    /// Wraps a [`DocumentSource`] (spec §5.16, §6.10).
    #[must_use]
    pub const fn document(source: DocumentSource) -> Self {
        Self::Document(source)
    }

    /// Wraps a [`GeneratedSource`] (spec §5.16).
    #[must_use]
    pub const fn generated(source: GeneratedSource) -> Self {
        Self::Generated(source)
    }

    /// Builds a file source from an opaque path string (spec §5.15 file).
    #[must_use]
    pub fn file(path: impl Into<Box<str>>) -> Self {
        Self::File(path.into())
    }

    /// Builds a URL source, resolved through a user/host loader (spec §5.15).
    #[must_use]
    pub fn url(url: impl Into<Box<str>>) -> Self {
        Self::Url(url.into())
    }

    /// Returns whether this is an [`ResourceSource::Embedded`] source.
    #[must_use]
    pub const fn is_embedded(&self) -> bool {
        matches!(self, Self::Embedded(_))
    }

    /// Returns whether this is a [`ResourceSource::Memory`] source.
    #[must_use]
    pub const fn is_memory(&self) -> bool {
        matches!(self, Self::Memory(_))
    }

    /// Returns whether this is a [`ResourceSource::Document`] source.
    #[must_use]
    pub const fn is_document(&self) -> bool {
        matches!(self, Self::Document(_))
    }

    /// Returns whether this is a [`ResourceSource::Generated`] source.
    #[must_use]
    pub const fn is_generated(&self) -> bool {
        matches!(self, Self::Generated(_))
    }

    /// Returns whether this is a [`ResourceSource::File`] source.
    #[must_use]
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    /// Returns whether this is a [`ResourceSource::Url`] source.
    #[must_use]
    pub const fn is_url(&self) -> bool {
        matches!(self, Self::Url(_))
    }

    /// Returns whether resolving this source requires a host loader outside the
    /// kit.
    ///
    /// True for [`ResourceSource::Url`] and [`ResourceSource::Document`]: the kit
    /// performs no network access and reads no host documents by itself (spec
    /// §5.15), so these must be handed to a user/host loader. All other kinds are
    /// either resolvable within the kit or by the loader port from a plain path.
    #[must_use]
    pub const fn requires_host_loader(&self) -> bool {
        matches!(self, Self::Url(_) | Self::Document(_))
    }

    /// Returns the bytes already in hand, if any.
    ///
    /// [`ResourceSource::Embedded`] and [`ResourceSource::Memory`] carry their
    /// bytes directly, so this returns `Some`. The file, URL, document, and
    /// generated kinds must be resolved before their bytes exist, so this returns
    /// `None`.
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
