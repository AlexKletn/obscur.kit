#![doc = include_str!("variant_image_decoder.doc/_module.md")]

use std::sync::Arc;

use obscur_geometry::Size;
use obscur_resources::handle::ResourceKind;
use obscur_resources::port::{DecodedResource, ResourceDecoder};
use obscur_resources::state::{ResourceError, ResourceMetadata};

#[doc = include_str!("variant_image_decoder.doc/VariantImageDecoder.md")]
#[derive(Debug, Clone)]
pub struct VariantImageDecoder;

impl ResourceDecoder for VariantImageDecoder {
    fn supports(&self, kind: ResourceKind) -> bool {
        kind == ResourceKind::Image
    }

    fn decode(&self, _kind: ResourceKind, bytes: &[u8]) -> Result<DecodedResource, ResourceError> {
        let n = u32::from(*bytes.first().ok_or(ResourceError::NotFound)?);
        let rgba: Arc<[u8]> = Arc::from(vec![255u8; (n * n * 4) as usize]);
        Ok(DecodedResource::new(
            rgba,
            ResourceMetadata::new().with_intrinsic_size(Size::new(n as f32, n as f32)),
        ))
    }
}
