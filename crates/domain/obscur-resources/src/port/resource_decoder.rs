#![doc = include_str!("resource_decoder.doc/_module.md")]

use std::sync::Arc;

use crate::handle::ResourceKind;
use crate::state::{ResourceError, ResourceMetadata};

#[doc = include_str!("resource_decoder.doc/DecodedResource.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct DecodedResource {
    data: Arc<[u8]>,
    metadata: ResourceMetadata,
}

impl DecodedResource {
    #[doc = include_str!("resource_decoder.doc/new.md")]
    #[must_use]
    pub fn new(data: Arc<[u8]>, metadata: ResourceMetadata) -> Self {
        Self { data, metadata }
    }

    #[doc = include_str!("resource_decoder.doc/data.md")]
    #[must_use]
    pub fn data(&self) -> &Arc<[u8]> {
        &self.data
    }

    #[doc = include_str!("resource_decoder.doc/metadata.md")]
    #[must_use]
    pub fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }
}

#[doc = include_str!("resource_decoder.doc/ResourceDecoder.md")]
pub trait ResourceDecoder {
    #[doc = include_str!("resource_decoder.doc/supports.md")]
    #[must_use]
    fn supports(&self, kind: ResourceKind) -> bool;

    #[doc = include_str!("resource_decoder.doc/decode.md")]
    fn decode(&self, kind: ResourceKind, bytes: &[u8]) -> Result<DecodedResource, ResourceError>;
}

#[doc = include_str!("resource_decoder.doc/NullDecoder.md")]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NullDecoder;

impl NullDecoder {
    #[doc = include_str!("resource_decoder.doc/new_2.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ResourceDecoder for NullDecoder {
    fn supports(&self, _kind: ResourceKind) -> bool {
        // Honest: a null decoder handles no kind.
        false
    }

    fn decode(&self, _kind: ResourceKind, _bytes: &[u8]) -> Result<DecodedResource, ResourceError> {
        Err(ResourceError::Unsupported("no decoder registered".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time proof the port is object-safe: adapters must be storable as
    /// `Box<dyn ResourceDecoder>`.
    #[test]
    fn resource_decoder_is_object_safe() {
        let decoder: Box<dyn ResourceDecoder> = Box::new(NullDecoder::new());
        assert!(!decoder.supports(ResourceKind::Image));
    }

    #[test]
    fn null_decoder_supports_nothing() {
        let decoder = NullDecoder::new();
        for kind in [
            ResourceKind::Image,
            ResourceKind::Font,
            ResourceKind::Stylesheet,
            ResourceKind::Shader,
            ResourceKind::Svg,
            ResourceKind::Atlas,
            ResourceKind::EmbeddedAsset,
        ] {
            assert!(!decoder.supports(kind));
        }
    }

    #[test]
    fn null_decoder_decode_is_unsupported() {
        let decoder = NullDecoder::new();
        assert_eq!(
            decoder.decode(ResourceKind::Image, &[1u8, 2, 3]),
            Err(ResourceError::Unsupported("no decoder registered".into()))
        );
    }

    #[test]
    fn decoded_resource_accessors() {
        let meta = ResourceMetadata::default().with_byte_size(4);
        let decoded = DecodedResource::new(Arc::from(&[0u8, 1, 2, 3][..]), meta.clone());
        assert_eq!(&**decoded.data(), &[0u8, 1, 2, 3]);
        assert_eq!(decoded.metadata(), &meta);
    }
}
