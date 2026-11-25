use std::sync::Arc;

use crate::source::ResourceSource;
use crate::state::{ResourceError, ResourceMetadata};

use super::{LoadedBytes, ResourceLoader};

#[doc = include_str!("../../docs/api/port/inline_loader.doc/InlineLoader.md")]
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
}
