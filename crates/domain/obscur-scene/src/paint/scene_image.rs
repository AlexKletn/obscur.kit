#![doc = include_str!("scene_image.doc/_module.md")]

use crate::color::AlphaMode;

use super::SharedPixels;

#[doc = include_str!("scene_image.doc/SceneImage.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct SceneImage {
    #[doc = include_str!("scene_image.doc/width.md")]
    width: u32,
    #[doc = include_str!("scene_image.doc/height.md")]
    height: u32,
    #[doc = include_str!("scene_image.doc/pixels.md")]
    pixels: SharedPixels,
    #[doc = include_str!("scene_image.doc/alpha.md")]
    alpha: AlphaMode,
    #[doc = include_str!("scene_image.doc/scale.md")]
    scale: f32,
}

impl SceneImage {
    #[doc = include_str!("scene_image.doc/new.md")]
    #[must_use]
    pub fn new(
        width: u32,
        height: u32,
        pixels: impl Into<SharedPixels>,
        alpha: AlphaMode,
    ) -> Option<Self> {
        let pixels = pixels.into();
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|px| px.checked_mul(4))?;
        if pixels.len() != expected {
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

    #[doc = include_str!("scene_image.doc/with_scale.md")]
    #[must_use]
    pub fn with_scale(mut self, scale: f32) -> Self {
        if scale.is_finite() && scale > 0.0 {
            self.scale = scale;
        }
        self
    }

    #[doc = include_str!("scene_image.doc/width_2.md")]
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    #[doc = include_str!("scene_image.doc/height_2.md")]
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    #[doc = include_str!("scene_image.doc/pixels_2.md")]
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        self.pixels.as_bytes()
    }

    #[doc = include_str!("scene_image.doc/shared_pixels.md")]
    #[must_use]
    pub fn shared_pixels(&self) -> SharedPixels {
        self.pixels.clone()
    }

    #[doc = include_str!("scene_image.doc/alpha_mode.md")]
    #[must_use]
    pub const fn alpha_mode(&self) -> AlphaMode {
        self.alpha
    }

    #[doc = include_str!("scene_image.doc/scale_2.md")]
    #[must_use]
    pub const fn scale(&self) -> f32 {
        self.scale
    }

    #[doc = include_str!("scene_image.doc/logical_size.md")]
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
