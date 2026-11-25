//! A one-pixel-per-logical-pixel frame in row-major premultiplied RGBA8.

use obscur_geometry::Size;

/// A one-pixel-per-logical-pixel frame in row-major premultiplied RGBA8.
#[derive(Debug, Clone)]
pub struct LayerPopupFrame {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl LayerPopupFrame {
    const BYTES_PER_PIXEL: u32 = 4;

    /// Returns the packed RGBA8 length for a supported frame size.
    #[must_use]
    pub fn byte_len_for_size(width_px: u32, height_px: u32) -> Option<usize> {
        if width_px == 0 || height_px == 0 {
            return None;
        }
        let stride_bytes = width_px.checked_mul(Self::BYTES_PER_PIXEL)?;
        let len_bytes = stride_bytes.checked_mul(height_px)?;
        // Keep the frame representable by native adapters with signed 32-bit buffer sizes.
        i32::try_from(len_bytes).ok()?;
        usize::try_from(len_bytes).ok()
    }

    /// Converts supported pixel dimensions to a one-to-one logical viewport.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn viewport_for_size(width_px: u32, height_px: u32) -> Option<Size> {
        Self::byte_len_for_size(width_px, height_px)?;
        let viewport = Size::new(width_px as f32, height_px as f32);
        // Layout rounds through f32, but native buffer dimensions must match exactly.
        (f64::from(viewport.width) == f64::from(width_px)
            && f64::from(viewport.height) == f64::from(height_px))
        .then_some(viewport)
    }

    /// A one-pixel-per-logical-pixel frame in row-major premultiplied RGBA8.
    #[must_use]
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Option<Self> {
        if pixels.len() != Self::byte_len_for_size(width, height)? {
            return None;
        }
        Some(Self {
            width,
            height,
            pixels,
        })
    }

    /// A one-pixel-per-logical-pixel frame in row-major premultiplied RGBA8.
    #[must_use]
    pub const fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Returns the number of bytes in a packed row.
    #[must_use]
    pub const fn row_stride_bytes(&self) -> u32 {
        self.width * Self::BYTES_PER_PIXEL
    }

    /// Returns row-major premultiplied RGBA8 pixels.
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_requires_exact_pixel_dimensions() {
        assert_eq!(
            LayerPopupFrame::viewport_for_size(32, 24),
            Some(Size::new(32.0, 24.0))
        );
        assert_eq!(LayerPopupFrame::viewport_for_size((1 << 24) + 1, 1), None);
        assert_eq!(LayerPopupFrame::viewport_for_size(1 << 29, 1), None);
    }

    #[test]
    fn byte_length_checks_empty_sizes_and_native_buffer_limits() {
        let max_width = i32::MAX as u32 / 4;
        assert_eq!(
            LayerPopupFrame::byte_len_for_size(max_width, 1),
            Some(2_147_483_644)
        );
        assert_eq!(LayerPopupFrame::byte_len_for_size(max_width + 1, 1), None);
        assert_eq!(LayerPopupFrame::byte_len_for_size(1 << 29, 2), None);
        assert_eq!(LayerPopupFrame::byte_len_for_size(1, 0), None);
    }

    #[test]
    fn frames_require_valid_dimensions_and_exact_pixel_data() {
        let frame = LayerPopupFrame::new(2, 3, vec![0; 24]).unwrap();
        assert_eq!(frame.row_stride_bytes(), 8);
        assert!(LayerPopupFrame::new(2, 1, vec![0; 7]).is_none());
        assert!(LayerPopupFrame::new(0, 1, Vec::new()).is_none());
        assert!(LayerPopupFrame::new(u32::MAX, 1, Vec::new()).is_none());
    }
}
