#![doc = include_str!("svg_decoder.doc/_module.md")]

use std::sync::Arc;

use obscur_geometry::Size;
use obscur_resources::handle::ResourceKind;
use obscur_resources::port::{DecodedResource, ResourceDecoder};
use obscur_resources::state::{ResourceError, ResourceMetadata};

#[doc = include_str!("svg_decoder.doc/SvgDecoder.md")]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SvgDecoder;

impl SvgDecoder {
    #[doc = include_str!("svg_decoder.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ResourceDecoder for SvgDecoder {
    fn supports(&self, kind: ResourceKind) -> bool {
        matches!(kind, ResourceKind::Svg)
    }

    fn decode(&self, kind: ResourceKind, bytes: &[u8]) -> Result<DecodedResource, ResourceError> {
        if kind != ResourceKind::Svg {
            return Err(ResourceError::Unsupported(
                "svg decoder handles only Svg resources".into(),
            ));
        }

        let options = usvg::Options::default();
        let tree = usvg::Tree::from_data(bytes, &options)
            .map_err(|err| ResourceError::DecodeFailed(err.to_string().into()))?;

        let size = tree.size();
        let metadata = ResourceMetadata::new()
            .with_byte_size(u64::try_from(bytes.len()).unwrap_or(u64::MAX))
            .with_mime_type("image/svg+xml")
            .with_intrinsic_size(Size::new(size.width(), size.height()));

        Ok(DecodedResource::new(Arc::from(bytes), metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SVG: &[u8] =
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20" viewBox="0 0 40 20"><rect width="40" height="20" fill="#0a0"/></svg>"##;

    #[test]
    fn supports_only_svg() {
        let decoder = SvgDecoder::new();
        assert!(decoder.supports(ResourceKind::Svg));
        for kind in [
            ResourceKind::Image,
            ResourceKind::Font,
            ResourceKind::Stylesheet,
            ResourceKind::Shader,
            ResourceKind::Atlas,
            ResourceKind::EmbeddedAsset,
        ] {
            assert!(!decoder.supports(kind));
        }
    }

    #[test]
    fn decodes_svg_keeping_source_and_intrinsic_size() {
        let decoded = SvgDecoder::new()
            .decode(ResourceKind::Svg, SAMPLE_SVG)
            .expect("svg decodes");
        // Vector-preserving: the payload is the original SVG source, unmodified.
        assert_eq!(&**decoded.data(), SAMPLE_SVG);
        assert_eq!(
            decoded.metadata().intrinsic_size(),
            Some(Size::new(40.0, 20.0))
        );
        assert_eq!(decoded.metadata().aspect_ratio(), Some(2.0));
        assert_eq!(decoded.metadata().mime_type(), Some("image/svg+xml"));
    }

    #[test]
    fn malformed_svg_is_decode_failed() {
        let decoder = SvgDecoder::new();
        let result = decoder.decode(ResourceKind::Svg, b"<not-svg>");
        assert!(matches!(result, Err(ResourceError::DecodeFailed(_))));
    }

    #[test]
    fn wrong_kind_is_unsupported() {
        let result = SvgDecoder::new().decode(ResourceKind::Image, SAMPLE_SVG);
        assert!(matches!(result, Err(ResourceError::Unsupported(_))));
    }
}
