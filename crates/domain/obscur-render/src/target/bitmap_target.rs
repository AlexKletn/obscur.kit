//! An offscreen render-to-CPU-bitmap destination (spec §5.2, §5.6).

use obscur_geometry::Size;

use crate::target::PixelFormat;

/// The domain descriptor of an offscreen render-to-CPU-bitmap destination
/// (spec §5.2, §5.6).
///
/// Software and headless backends render into a CPU bitmap here — this is the
/// destination for golden-image tests, where the deterministic scene contract of
/// spec §5.2 is compared pixel-for-pixel. The optional [`Self::row_stride`]
/// records the number of bytes between the start of consecutive rows; `None`
/// means the rows are tightly packed at `width * bytes_per_pixel`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BitmapTarget {
    /// The bitmap size in physical pixels.
    pub size: Size,
    /// The bitmap's pixel format.
    pub format: PixelFormat,
    /// The number of bytes between the start of consecutive rows, or `None` when
    /// rows are tightly packed.
    pub row_stride: Option<u32>,
}

impl BitmapTarget {
    /// Creates a bitmap target of the given `size`, defaulting to
    /// [`PixelFormat::Rgba8Unorm`] and tightly-packed rows (`row_stride` `None`).
    #[must_use]
    pub const fn new(size: Size) -> Self {
        Self {
            size,
            format: PixelFormat::Rgba8Unorm,
            row_stride: None,
        }
    }

    /// Sets the pixel format, returning the updated target.
    #[must_use]
    pub const fn with_format(mut self, format: PixelFormat) -> Self {
        self.format = format;
        self
    }

    /// Sets an explicit row stride in bytes, returning the updated target.
    #[must_use]
    pub const fn with_row_stride(mut self, row_stride: u32) -> Self {
        self.row_stride = Some(row_stride);
        self
    }

    /// The bitmap size in physical pixels.
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    /// The bitmap's pixel format.
    #[must_use]
    pub const fn format(&self) -> PixelFormat {
        self.format
    }

    /// The explicit row stride in bytes, or `None` when rows are tightly packed.
    #[must_use]
    pub const fn row_stride(&self) -> Option<u32> {
        self.row_stride
    }

    /// The effective row stride in bytes: the explicit [`Self::row_stride`] if
    /// set, otherwise the tightly-packed stride `width * bytes_per_pixel`.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn effective_row_stride(&self) -> u32 {
        match self.row_stride {
            Some(stride) => stride,
            None => (self.size.width as u32) * u32::from(self.format.bytes_per_pixel()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_defaults_to_rgba8_packed() {
        let target = BitmapTarget::new(Size::new(64.0, 32.0));
        assert_eq!(target.size(), Size::new(64.0, 32.0));
        assert_eq!(target.format(), PixelFormat::Rgba8Unorm);
        assert_eq!(target.row_stride(), None);
    }

    #[test]
    fn builders_override_fields() {
        let target = BitmapTarget::new(Size::new(1.0, 1.0))
            .with_format(PixelFormat::Rgba16Float)
            .with_row_stride(512);
        assert_eq!(target.format(), PixelFormat::Rgba16Float);
        assert_eq!(target.row_stride(), Some(512));
    }

    #[test]
    fn effective_row_stride_packed_vs_explicit() {
        // Packed: 64 px * 4 bytes/px = 256.
        let packed = BitmapTarget::new(Size::new(64.0, 32.0));
        assert_eq!(packed.effective_row_stride(), 256);

        // Rgba16Float is 8 bytes/px: 64 * 8 = 512.
        let hdr = BitmapTarget::new(Size::new(64.0, 32.0)).with_format(PixelFormat::Rgba16Float);
        assert_eq!(hdr.effective_row_stride(), 512);

        // Explicit stride wins over the packed computation.
        let explicit = BitmapTarget::new(Size::new(64.0, 32.0)).with_row_stride(300);
        assert_eq!(explicit.effective_row_stride(), 300);
    }
}
