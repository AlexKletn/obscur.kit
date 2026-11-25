/// The pixel format of a [`RenderTarget`](super::RenderTarget)'s color buffer (spec §5.2).
///
/// The determinism boundary of the renderer abstraction (spec §5.2) fixes
/// geometry and paint order across backends but allows precision to differ; the
/// format chosen here is part of that precision contract, ranging from the
/// common 8-bit-per-channel surface formats to a 16-bit float format for HDR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// 8-bit-per-channel RGBA, unsigned normalized. The common offscreen format.
    Rgba8Unorm,
    /// 8-bit-per-channel BGRA, unsigned normalized. The common window format.
    Bgra8Unorm,
    /// 16-bit-per-channel RGBA, floating point. Used for HDR output.
    Rgba16Float,
}

impl PixelFormat {
    /// Whether this format carries high-dynamic-range (HDR) color.
    ///
    /// True exactly for [`PixelFormat::Rgba16Float`].
    #[must_use]
    pub const fn is_hdr(&self) -> bool {
        matches!(self, Self::Rgba16Float)
    }

    /// The number of bytes one pixel occupies in this format.
    ///
    /// The 8-bit formats use 4 bytes per pixel; the 16-bit float format uses 8.
    #[must_use]
    pub const fn bytes_per_pixel(&self) -> u8 {
        match self {
            Self::Rgba8Unorm | Self::Bgra8Unorm => 4,
            Self::Rgba16Float => 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_format_hdr_only_for_rgba16_float() {
        assert!(!PixelFormat::Rgba8Unorm.is_hdr());
        assert!(!PixelFormat::Bgra8Unorm.is_hdr());
        assert!(PixelFormat::Rgba16Float.is_hdr());
    }

    #[test]
    fn pixel_format_bytes_per_pixel() {
        assert_eq!(PixelFormat::Rgba8Unorm.bytes_per_pixel(), 4);
        assert_eq!(PixelFormat::Bgra8Unorm.bytes_per_pixel(), 4);
        assert_eq!(PixelFormat::Rgba16Float.bytes_per_pixel(), 8);
    }
}
