use crate::state::ResourceMetadata;

#[doc = include_str!("../../docs/api/port/compiled_resource.doc/CompiledResource.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledResource {
    byte_size: u64,
    metadata: ResourceMetadata,
}

impl CompiledResource {
    /// Creates a compiled-resource descriptor from the artifact's backend/GPU
    /// `byte_size` and its `metadata`.
    #[must_use]
    pub fn new(byte_size: u64, metadata: ResourceMetadata) -> Self {
        Self {
            byte_size,
            metadata,
        }
    }

    /// Returns the backend/GPU footprint in bytes, used for cache budgeting (spec
    /// §5.15).
    #[must_use]
    pub const fn byte_size(&self) -> u64 {
        self.byte_size
    }

    /// Returns the metadata carried through compilation (spec §5.15).
    #[must_use]
    pub fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_resource_accessors() {
        let meta = ResourceMetadata::default().with_byte_size(64);
        let compiled = CompiledResource::new(256, meta.clone());
        assert_eq!(compiled.byte_size(), 256);
        assert_eq!(compiled.metadata(), &meta);
    }
}
