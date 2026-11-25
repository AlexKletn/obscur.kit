//! A decoded raster image, ready to be drawn by a renderer.

use crate::color::AlphaMode;

use super::SharedPixels;

#[doc = include_str!("../../docs/api/paint/scene_image.doc/SceneImage.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct SceneImage {
    /// The image width in pixels (its intrinsic width).
    width: u32,
    /// The image height in pixels (its intrinsic height).
    height: u32,
    /// `width × height × 4` RGBA8 bytes, row-major, top-to-bottom. Shared so a
    /// clone is cheap.
    pixels: SharedPixels,
    /// Whether the RGBA channels are premultiplied by alpha.
    alpha: AlphaMode,
    /// The asset's device-pixel density (spec §5.16 DPI variants): how many of
    /// this image's pixels span one logical CSS pixel. `1.0` for a plain asset;
    /// `2.0` for an `@2x` variant, whose pixel dimensions are twice its logical
    /// size.
    scale: f32,
}

impl SceneImage {
    /// Returns the RGBA8 payload size in bytes when it fits in one allocation.
    #[must_use]
    pub fn rgba8_byte_size(width_px: u32, height_px: u32) -> Option<u64> {
        const BYTES_PER_PIXEL: u64 = 4;
        let bytes = u64::from(width_px)
            .checked_mul(u64::from(height_px))?
            .checked_mul(BYTES_PER_PIXEL)?;
        isize::try_from(bytes).ok().map(|_| bytes)
    }

    #[doc = include_str!("../../docs/api/paint/scene_image.doc/new.md")]
    #[must_use]
    pub fn new(
        width: u32,
        height: u32,
        pixels: impl Into<SharedPixels>,
        alpha: AlphaMode,
    ) -> Option<Self> {
        let expected = Self::rgba8_byte_size(width, height)?;
        let pixels = pixels.into();
        if u64::try_from(pixels.len()).ok()? != expected {
            return None;
        }
        Some(Self {
            width,
            height,
            pixels,
            alpha,
            scale: 1.0,
        })
    }

    #[doc = include_str!("../../docs/api/paint/scene_image.doc/with_scale.md")]
    #[must_use]
    pub fn with_scale(mut self, scale: f32) -> Self {
        if scale.is_finite() && scale > 0.0 {
            self.scale = scale;
        }
        self
    }

    /// The image's intrinsic width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// The image's intrinsic height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// The RGBA8 pixel bytes, row-major and top-to-bottom (`width × height × 4`).
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        self.pixels.as_bytes()
    }

    #[doc = include_str!("../../docs/api/paint/scene_image.doc/shared_pixels.md")]
    #[must_use]
    pub fn shared_pixels(&self) -> SharedPixels {
        self.pixels.clone()
    }

    /// Whether the RGBA channels are premultiplied by alpha.
    #[must_use]
    pub const fn alpha_mode(&self) -> AlphaMode {
        self.alpha
    }

    /// The asset's device-pixel density: how many of this image's pixels span one
    /// logical CSS pixel (spec §5.16 DPI variants). `1.0` unless
    /// [`with_scale`](Self::with_scale) tagged it.
    #[must_use]
    pub const fn scale(&self) -> f32 {
        self.scale
    }

    #[doc = include_str!("../../docs/api/paint/scene_image.doc/logical_size.md")]
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn logical_size(&self) -> (f32, f32) {
        (
            self.width as f32 / self.scale,
            self.height as f32 / self.scale,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// A 2×2 image: opaque red, opaque green, opaque blue, opaque white.
    fn rgba_2x2() -> Vec<u8> {
        vec![
            255, 0, 0, 255, // red
            0, 255, 0, 255, // green
            0, 0, 255, 255, // blue
            255, 255, 255, 255, // white
        ]
    }

    #[test]
    fn constructs_from_a_2x2_buffer() {
        let img = SceneImage::new(2, 2, Arc::from(rgba_2x2()), AlphaMode::Straight)
            .expect("2x2 * 4 bytes is a valid image");
        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 2);
        assert_eq!(img.alpha_mode(), AlphaMode::Straight);
        assert_eq!(img.pixels().len(), 16);
        // First pixel is opaque red.
        assert_eq!(&img.pixels()[0..4], &[255, 0, 0, 255]);
    }

    #[test]
    fn rgba_size_rejects_overflow_and_sizes_outside_allocation_range() {
        assert_eq!(SceneImage::rgba8_byte_size(3, 2), Some(24));
        assert_eq!(SceneImage::rgba8_byte_size(u32::MAX, u32::MAX), None);
        assert_eq!(SceneImage::rgba8_byte_size(u32::MAX, 1 << 30), None);
    }

    #[test]
    fn wrong_byte_length_is_rejected() {
        // 2×2 needs 16 bytes; 15 is short and 17 is long — both rejected.
        assert!(SceneImage::new(2, 2, Arc::from(vec![0u8; 15]), AlphaMode::Straight).is_none());
        assert!(SceneImage::new(2, 2, Arc::from(vec![0u8; 17]), AlphaMode::Straight).is_none());
        // An exact match is accepted.
        assert!(SceneImage::new(2, 2, Arc::from(vec![0u8; 16]), AlphaMode::Straight).is_some());
    }

    #[test]
    fn zero_sized_image_needs_zero_bytes() {
        assert!(SceneImage::new(0, 0, Arc::from(Vec::new()), AlphaMode::Straight).is_some());
        assert!(SceneImage::new(0, 0, Arc::from(vec![0u8; 4]), AlphaMode::Straight).is_none());
    }

    #[test]
    fn shared_pixels_shares_the_same_allocation() {
        let img = SceneImage::new(2, 2, Arc::from(rgba_2x2()), AlphaMode::Premultiplied)
            .expect("valid image");
        assert!(std::ptr::eq(
            img.shared_pixels().as_bytes().as_ptr(),
            img.pixels().as_ptr()
        ));
        assert_eq!(img.alpha_mode(), AlphaMode::Premultiplied);
    }

    #[test]
    fn a_plain_image_is_base_density_and_logical_size_is_the_pixel_size() {
        let img =
            SceneImage::new(2, 2, Arc::from(rgba_2x2()), AlphaMode::Straight).expect("valid image");
        assert_eq!(img.scale(), 1.0);
        assert_eq!(img.logical_size(), (2.0, 2.0));
    }

    #[test]
    fn a_2x_variant_lays_out_at_half_its_pixel_size() {
        let img = SceneImage::new(2, 2, Arc::from(rgba_2x2()), AlphaMode::Straight)
            .expect("valid image")
            .with_scale(2.0);
        // The pixel dimensions are untouched — the renderer still blits 2×2 …
        assert_eq!((img.width(), img.height()), (2, 2));
        assert_eq!(img.scale(), 2.0);
        // … but it stands in for a 1×1 logical box.
        assert_eq!(img.logical_size(), (1.0, 1.0));
    }

    #[test]
    fn a_degenerate_scale_falls_back_to_the_base_density() {
        let base = || {
            SceneImage::new(2, 2, Arc::from(rgba_2x2()), AlphaMode::Straight).expect("valid image")
        };
        // Each of these could only produce a degenerate (infinite/NaN) logical
        // size, so the tag is refused rather than honored.
        assert_eq!(base().with_scale(0.0).scale(), 1.0);
        assert_eq!(base().with_scale(-2.0).scale(), 1.0);
        assert_eq!(base().with_scale(f32::NAN).scale(), 1.0);
        assert_eq!(base().with_scale(f32::INFINITY).scale(), 1.0);
        assert_eq!(base().with_scale(0.0).logical_size(), (2.0, 2.0));
    }

    #[test]
    fn clone_is_equal_and_shares_pixels() {
        let img =
            SceneImage::new(2, 2, Arc::from(rgba_2x2()), AlphaMode::Straight).expect("valid image");
        let clone = img.clone();
        assert_eq!(img, clone);
        // Cloning shares the pixel allocation (cheap clone).
        assert!(std::ptr::eq(img.pixels().as_ptr(), clone.pixels().as_ptr()));
    }
}
