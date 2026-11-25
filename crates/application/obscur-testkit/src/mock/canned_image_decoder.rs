//! A decoder that always answers the same way.

use std::sync::Arc;

use obscur_geometry::Size;
use obscur_resources::handle::ResourceKind;
use obscur_resources::port::{DecodedResource, ResourceDecoder};
use obscur_resources::state::{ResourceError, ResourceMetadata};

#[doc = include_str!("../../docs/api/mock/canned_image_decoder.doc/CannedImageDecoder.md")]
#[derive(Debug, Clone)]
pub struct CannedImageDecoder;

impl ResourceDecoder for CannedImageDecoder {
    fn supports(&self, kind: ResourceKind) -> bool {
        kind == ResourceKind::Image
    }

    fn decode(&self, _kind: ResourceKind, _bytes: &[u8]) -> Result<DecodedResource, ResourceError> {
        let rgba: Arc<[u8]> = Arc::from(vec![255u8; 2 * 2 * 4]);
        Ok(DecodedResource::new(
            rgba,
            ResourceMetadata::new().with_intrinsic_size(Size::new(2.0, 2.0)),
        ))
    }
}
