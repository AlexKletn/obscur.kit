use std::sync::Arc;

use crate::state::ResourceMetadata;

#[doc = include_str!("../../docs/api/port/decoded_resource.doc/DecodedResource.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct DecodedResource {
    data: Arc<[u8]>,
    metadata: ResourceMetadata,
}

impl DecodedResource {
    /// Creates a decoded resource from CPU `data` and its `metadata`.
    #[must_use]
    pub fn new(data: Arc<[u8]>, metadata: ResourceMetadata) -> Self {
        Self { data, metadata }
    }

    /// Returns the decoded CPU bytes (spec §5.15).
    #[must_use]
    pub fn data(&self) -> &Arc<[u8]> {
        &self.data
    }

    /// Returns the metadata determined while decoding (spec §5.15), including the
    /// intrinsic size for a visual resource.
    #[must_use]
    pub fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoded_resource_accessors() {
        let meta = ResourceMetadata::default().with_byte_size(4);
        let decoded = DecodedResource::new(Arc::from(&[0u8, 1, 2, 3][..]), meta.clone());
        assert_eq!(&**decoded.data(), &[0u8, 1, 2, 3]);
        assert_eq!(decoded.metadata(), &meta);
    }
}
