//! A decoded raster image, ready to be drawn by a renderer.

use std::sync::Arc;

use crate::color::AlphaMode;

/// A decoded raster image: `width × height` RGBA8 pixels plus how their alpha is
/// stored (spec §5.16).
///
/// This is the backend-independent value a
/// [`PaintCommand::DrawImage`](crate::command::PaintCommand::DrawImage) carries.
/// It holds only **raw bytes** — the domain never depends on an image codec or a
/// GPU/CPU image type — so a renderer is free to wrap the pixels into its own
/// texture (`peniko::Image`, `tiny_skia::Pixmap`, …). Pixels are laid out row by
/// row, top-to-bottom, four bytes (`R`, `G`, `B`, `A`) per pixel; the
/// [`AlphaMode`] states whether those channels are premultiplied by alpha.
///
/// The pixel buffer is stored behind an [`Arc`] so cloning a `SceneImage` — which
/// happens every time a display list is cloned — only bumps a refcount rather than
/// copying the (potentially large) pixel data.
///
/// # Not yet modeled
///
/// A `SceneImage` describes a whole decoded bitmap drawn at a destination rect
/// (see [`PaintCommand::DrawImage`](crate::command::PaintCommand::DrawImage)). A
/// source sub-rectangle, tiling/`repeat`, nine-slice, and CSS `background-size`
/// keywords are future additions; today an image is stretched to fill its
/// destination rect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneImage {
    /// The image width in pixels (its intrinsic width).
    width: u32,
    /// The image height in pixels (its intrinsic height).
    height: u32,
    /// `width × height × 4` RGBA8 bytes, row-major, top-to-bottom. Shared so a
    /// clone is cheap.
    pixels: Arc<[u8]>,
    /// Whether the RGBA channels are premultiplied by alpha.
    alpha: AlphaMode,
}

impl SceneImage {
    /// Creates an image from `width × height` RGBA8 `pixels` with the given
    /// [`AlphaMode`], or `None` when the buffer length does not equal
    /// `width × height × 4` (the byte-length guard: four bytes per pixel).
    ///
    /// The pixels are taken as an [`Arc`] so no copy is made; callers with an
    /// owned `Vec<u8>` can pass `Arc::from(bytes)`.
    #[must_use]
    pub fn new(width: u32, height: u32, pixels: Arc<[u8]>, alpha: AlphaMode) -> Option<Self> {
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
        })
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
        &self.pixels
    }

    /// A cheap [`Arc`] handle to the shared pixel buffer (bumps a refcount rather
    /// than copying). A renderer wraps this into its own image type.
    #[must_use]
    pub fn pixels_arc(&self) -> Arc<[u8]> {
        Arc::clone(&self.pixels)
    }

    /// Whether the RGBA channels are premultiplied by alpha.
    #[must_use]
    pub const fn alpha_mode(&self) -> AlphaMode {
        self.alpha
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn pixels_arc_shares_the_same_allocation() {
        let img = SceneImage::new(2, 2, Arc::from(rgba_2x2()), AlphaMode::Premultiplied)
            .expect("valid image");
        let handle = img.pixels_arc();
        assert!(Arc::ptr_eq(&handle, &img.pixels_arc()));
        assert_eq!(img.alpha_mode(), AlphaMode::Premultiplied);
    }

    #[test]
    fn clone_is_equal_and_shares_pixels() {
        let img = SceneImage::new(2, 2, Arc::from(rgba_2x2()), AlphaMode::Straight)
            .expect("valid image");
        let clone = img.clone();
        assert_eq!(img, clone);
        // Cloning shares the pixel allocation (cheap clone).
        assert!(Arc::ptr_eq(&img.pixels_arc(), &clone.pixels_arc()));
    }
}
