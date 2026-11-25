//! The decoder that reads a variant's pixel size back.

use std::sync::Arc;

use obscur_geometry::Size;
use obscur_resources::handle::ResourceKind;
use obscur_resources::port::{DecodedResource, ResourceDecoder};
use obscur_resources::state::{ResourceError, ResourceMetadata};

/// [`VariantImageLoader`](crate::mock::VariantImageLoader)'s counterpart: decodes
/// the one-byte "asset" into an `N×N` opaque RGBA image, so the decoded pixel size
/// names the file it came from.
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
