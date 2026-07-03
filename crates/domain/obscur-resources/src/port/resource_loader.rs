//! The hexagonal port for loading a resource's raw encoded bytes (spec §5.15).
//!
//! This module defines [`ResourceLoader`], the port through which the pure
//! resource domain obtains the raw *encoded* bytes behind a [`ResourceSource`]
//! (spec §5.15 custom loaders). Interface adapters implement it — an images/font
//! crate that opens files, a host loader that fetches URLs or reads picked
//! documents — while the kit itself performs no network access and reads no host
//! documents by itself. The colocated [`LoadedBytes`] output pairs the encoded
//! bytes with their [`ResourceMetadata`]; the colocated [`InlineLoader`] is an
//! honest reference implementation that resolves only the bytes-in-hand sources.
//!
//! Loading is modeled **synchronously** at the domain level: an adapter that
//! loads asynchronously wraps its own async and calls this port when the bytes
//! are ready.

use std::sync::Arc;

use crate::source::ResourceSource;
use crate::state::{ResourceError, ResourceMetadata};

/// The raw encoded bytes of a resource plus its descriptive metadata (spec
/// §5.15).
///
/// This is the output of [`ResourceLoader::load`]: the still-encoded bytes (a
/// PNG blob, a font file, a CSS string) held in an [`Arc<[u8]>`] so clones are
/// cheap and the bytes can be shared across the decode/compile pipeline, paired
/// with the [`ResourceMetadata`] the loader could determine (at minimum the byte
/// size).
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedBytes {
    bytes: Arc<[u8]>,
    metadata: ResourceMetadata,
}

impl LoadedBytes {
    /// Creates a loaded-bytes value from encoded `bytes` and their `metadata`.
    #[must_use]
    pub fn new(bytes: Arc<[u8]>, metadata: ResourceMetadata) -> Self {
        Self { bytes, metadata }
    }

    /// Returns the raw encoded bytes (spec §5.15).
    #[must_use]
    pub fn bytes(&self) -> &Arc<[u8]> {
        &self.bytes
    }

    /// Returns the metadata determined while loading (spec §5.15).
    #[must_use]
    pub fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }
}

/// The hexagonal port for loading a resource's raw encoded bytes (spec §5.15
/// custom loaders).
///
/// An adapter implements this once per loading strategy, and the domain stores
/// it as `Box<dyn ResourceLoader>`; the trait is therefore deliberately
/// **object-safe** — no generic methods and no `Self`-returning methods — so
/// loaders can be held polymorphically and composed. A loader turns the pure
/// *description* of a [`ResourceSource`] into the encoded bytes behind it,
/// reporting failure through [`ResourceError`] rather than panicking.
pub trait ResourceLoader {
    /// Returns whether this loader can resolve `source` (spec §5.15).
    ///
    /// A caller consults this to route a source to a loader that understands it
    /// before calling [`Self::load`].
    #[must_use]
    fn can_load(&self, source: &ResourceSource) -> bool;

    /// Loads the raw encoded bytes behind `source` (spec §5.15).
    ///
    /// Returns [`LoadedBytes`] on success, or a [`ResourceError`] describing why
    /// the source could not be resolved — for example
    /// [`ResourceError::SourceUnavailable`] when this loader cannot reach the
    /// source at all.
    fn load(&self, source: &ResourceSource) -> Result<LoadedBytes, ResourceError>;
}

/// A reference [`ResourceLoader`] that resolves only the bytes-in-hand sources
/// (spec §5.15).
///
/// `InlineLoader` serves the [`ResourceSource::Embedded`] and
/// [`ResourceSource::Memory`] kinds, whose bytes are already present, by copying
/// them into an [`Arc<[u8]>`]. It cannot resolve file, URL, document, or
/// generated sources — those require a host loader — so for them it honestly
/// reports [`ResourceLoader::can_load`] `== false` and returns
/// [`ResourceError::SourceUnavailable`] from [`ResourceLoader::load`]. Its
/// refusals are correct behavior, not stubs waiting to be filled in.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct InlineLoader;

impl InlineLoader {
    /// Creates an [`InlineLoader`].
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ResourceLoader for InlineLoader {
    fn can_load(&self, source: &ResourceSource) -> bool {
        source.inline_bytes().is_some()
    }

    fn load(&self, source: &ResourceSource) -> Result<LoadedBytes, ResourceError> {
        match source.inline_bytes() {
            Some(bytes) => {
                let len = bytes.len() as u64;
                Ok(LoadedBytes::new(
                    Arc::from(bytes),
                    ResourceMetadata::default().with_byte_size(len),
                ))
            }
            None => Err(ResourceError::SourceUnavailable(
                "InlineLoader serves only Embedded/Memory sources; a host loader must \
                 resolve file, URL, document, and generated sources"
                    .into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{EmbeddedSource, MemorySource};

    /// Compile-time proof the port is object-safe: adapters must be storable as
    /// `Box<dyn ResourceLoader>`.
    #[test]
    fn resource_loader_is_object_safe() {
        let loader: Box<dyn ResourceLoader> = Box::new(InlineLoader::new());
        assert!(!loader.can_load(&ResourceSource::file("/x")));
    }

    #[test]
    fn loads_memory_source() {
        let loader = InlineLoader::new();
        let source = ResourceSource::memory(MemorySource::new(b"hello".to_vec()));
        assert!(loader.can_load(&source));
        let loaded = loader.load(&source).expect("memory source loads");
        assert_eq!(&**loaded.bytes(), b"hello");
        assert_eq!(loaded.metadata().byte_size(), Some(5));
    }

    #[test]
    fn loads_embedded_source() {
        let loader = InlineLoader::new();
        let source = ResourceSource::embedded(EmbeddedSource::new("logo", vec![1u8, 2, 3]));
        assert!(loader.can_load(&source));
        let loaded = loader.load(&source).expect("embedded source loads");
        assert_eq!(&**loaded.bytes(), &[1u8, 2, 3]);
        assert_eq!(loaded.metadata().byte_size(), Some(3));
    }

    #[test]
    fn rejects_file_source_honestly() {
        let loader = InlineLoader::new();
        let source = ResourceSource::file("/path/to/asset.png");
        assert!(!loader.can_load(&source));
        assert!(matches!(
            loader.load(&source),
            Err(ResourceError::SourceUnavailable(_))
        ));
    }

    #[test]
    fn rejects_url_source_honestly() {
        let loader = InlineLoader::new();
        let source = ResourceSource::url("https://example.com/a.png");
        assert!(!loader.can_load(&source));
        assert!(matches!(
            loader.load(&source),
            Err(ResourceError::SourceUnavailable(_))
        ));
    }

    #[test]
    fn loaded_bytes_accessors() {
        let meta = ResourceMetadata::default().with_byte_size(2);
        let loaded = LoadedBytes::new(Arc::from(&[9u8, 8][..]), meta.clone());
        assert_eq!(&**loaded.bytes(), &[9u8, 8]);
        assert_eq!(loaded.metadata(), &meta);
    }
}
