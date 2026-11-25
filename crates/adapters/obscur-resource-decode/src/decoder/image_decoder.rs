#![doc = include_str!("../../docs/api/decoder/image_decoder.doc/_module.md")]

use std::{io::Cursor, sync::Arc};

use obscur_geometry::Size;
use obscur_resources::handle::ResourceKind;
use obscur_resources::port::{DecodedResource, ResourceDecoder};
use obscur_resources::state::{ResourceError, ResourceMetadata};
use obscur_scene::paint::SceneImage;

/// The default maximum per-side pixel dimension a decoded image may have (spec
/// §5.16 safety limits).
const DEFAULT_MAX_DIMENSION: u32 = 16_384;

#[doc = include_str!("../../docs/api/decoder/image_decoder.doc/ImageDecoder.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageDecoder {
    /// The maximum allowed width or height, in pixels, before a decode is
    /// rejected as [`ResourceError::LimitExceeded`] (spec §5.16).
    max_dimension: u32,
}

impl ImageDecoder {
    /// Creates an image decoder with the default per-side dimension limit
    /// ([`DEFAULT_MAX_DIMENSION`]).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_dimension: DEFAULT_MAX_DIMENSION,
        }
    }

    /// Creates an image decoder that rejects any image wider or taller than
    /// `max_dimension` pixels (spec §5.16 safety limits).
    #[must_use]
    pub const fn with_max_dimension(max_dimension: u32) -> Self {
        Self { max_dimension }
    }

    /// Returns the current per-side dimension limit in pixels.
    #[must_use]
    pub const fn max_dimension(&self) -> u32 {
        self.max_dimension
    }
}

impl Default for ImageDecoder {
    /// The default image decoder uses [`ImageDecoder::new`].
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceDecoder for ImageDecoder {
    fn supports(&self, kind: ResourceKind) -> bool {
        matches!(kind, ResourceKind::Image)
    }

    fn decode(&self, kind: ResourceKind, bytes: &[u8]) -> Result<DecodedResource, ResourceError> {
        if kind != ResourceKind::Image {
            return Err(ResourceError::Unsupported(
                "image decoder handles only Image resources".into(),
            ));
        }

        let mut reader = image::ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|error| ResourceError::DecodeFailed(error.to_string().into()))?;
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(self.max_dimension);
        limits.max_image_height = Some(self.max_dimension);
        reader.limits(limits);
        let image = reader.decode().map_err(|error| match error {
            image::ImageError::Limits(_) => ResourceError::LimitExceeded(error.to_string().into()),
            _ => ResourceError::DecodeFailed(error.to_string().into()),
        })?;
        let width = image.width();
        let height = image.height();

        let rgba_bytes = SceneImage::rgba8_byte_size(width, height).ok_or_else(|| {
            ResourceError::LimitExceeded(
                format!("RGBA buffer for image {width}x{height} exceeds addressable size").into(),
            )
        })?;
        let rgba = image.into_rgba8().into_raw();
        let metadata = ResourceMetadata::new()
            .with_byte_size(rgba_bytes)
            .with_mime_type("image/rgba8")
            .with_intrinsic_size(Size::new(width as f32, height as f32));

        Ok(DecodedResource::new(Arc::from(rgba), metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Encodes a solid-color RGBA image to the given format, returning the
    /// encoded bytes — a real fixture round-tripped through the `image` crate.
    fn encode(width: u32, height: u32, format: image::ImageFormat) -> Vec<u8> {
        let buffer = image::RgbaImage::from_pixel(width, height, image::Rgba([10, 200, 30, 255]));
        let dynamic = image::DynamicImage::ImageRgba8(buffer);
        let mut out = Cursor::new(Vec::new());
        dynamic.write_to(&mut out, format).expect("encode fixture");
        out.into_inner()
    }

    #[test]
    fn supports_only_image() {
        let decoder = ImageDecoder::new();
        assert!(decoder.supports(ResourceKind::Image));
        for kind in [
            ResourceKind::Font,
            ResourceKind::Svg,
            ResourceKind::Stylesheet,
            ResourceKind::Shader,
            ResourceKind::Atlas,
            ResourceKind::EmbeddedAsset,
        ] {
            assert!(!decoder.supports(kind));
        }
    }

    #[test]
    fn decodes_png_to_rgba_with_intrinsic_size() {
        let png = encode(3, 2, image::ImageFormat::Png);
        let decoded = ImageDecoder::new()
            .decode(ResourceKind::Image, &png)
            .expect("png decodes");

        // 3x2 pixels * 4 bytes each.
        assert_eq!(decoded.data().len(), 3 * 2 * 4);
        assert_eq!(decoded.metadata().byte_size(), Some(24));
        assert_eq!(
            decoded.metadata().intrinsic_size(),
            Some(Size::new(3.0, 2.0))
        );
        assert_eq!(decoded.metadata().aspect_ratio(), Some(1.5));
        // The first pixel is the solid color we encoded.
        assert_eq!(&decoded.data()[0..4], &[10, 200, 30, 255]);
    }

    #[test]
    fn decodes_jpeg_dimensions() {
        let jpeg = encode(4, 4, image::ImageFormat::Jpeg);
        let decoded = ImageDecoder::new()
            .decode(ResourceKind::Image, &jpeg)
            .expect("jpeg decodes");
        assert_eq!(
            decoded.metadata().intrinsic_size(),
            Some(Size::new(4.0, 4.0))
        );
        assert_eq!(decoded.data().len(), 4 * 4 * 4);
    }

    #[test]
    fn malformed_bytes_are_decode_failed() {
        let decoder = ImageDecoder::new();
        let result = decoder.decode(ResourceKind::Image, &[0u8, 1, 2, 3, 4]);
        assert!(matches!(result, Err(ResourceError::DecodeFailed(_))));
    }

    #[test]
    fn oversized_image_hits_limit() {
        let png = encode(3, 2, image::ImageFormat::Png);
        let decoder = ImageDecoder::with_max_dimension(2);
        let result = decoder.decode(ResourceKind::Image, &png);
        assert!(matches!(result, Err(ResourceError::LimitExceeded(_))));
    }

    #[test]
    fn rejects_oversized_header_before_reading_invalid_pixel_data() {
        let mut png = encode(3, 2, image::ImageFormat::Png);
        let idat = png.windows(4).position(|tag| tag == b"IDAT").unwrap();
        png[idat + 4] ^= 0xff;
        assert!(matches!(
            ImageDecoder::with_max_dimension(2).decode(ResourceKind::Image, &png),
            Err(ResourceError::LimitExceeded(_))
        ));
        assert!(matches!(
            ImageDecoder::with_max_dimension(3).decode(ResourceKind::Image, &png),
            Err(ResourceError::DecodeFailed(_))
        ));
    }

    #[test]
    fn wrong_kind_is_unsupported() {
        let png = encode(1, 1, image::ImageFormat::Png);
        let result = ImageDecoder::new().decode(ResourceKind::Font, &png);
        assert!(matches!(result, Err(ResourceError::Unsupported(_))));
    }
}
