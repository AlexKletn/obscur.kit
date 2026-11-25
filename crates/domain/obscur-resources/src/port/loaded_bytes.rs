use std::sync::Arc;

use crate::state::ResourceMetadata;

#[doc = include_str!("../../docs/api/port/loaded_bytes.doc/LoadedBytes.md")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loaded_bytes_accessors() {
        let meta = ResourceMetadata::default().with_byte_size(2);
        let loaded = LoadedBytes::new(Arc::from(&[9u8, 8][..]), meta.clone());
        assert_eq!(&**loaded.bytes(), &[9u8, 8]);
        assert_eq!(loaded.metadata(), &meta);
    }
}
