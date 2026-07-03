//! An offscreen render-to-GPU-texture destination (spec §5.6).

use obscur_geometry::Size;

use crate::target::PixelFormat;

/// The domain descriptor of an offscreen render-to-GPU-texture destination
/// (spec §5.6).
///
/// Offscreen output renders into a GPU texture rather than a window surface. The
/// [`Self::sampled`] flag is a hint that the rendered result will subsequently
/// be read back as a sampled texture (for example, composited into another
/// scene), which a backend may use when allocating the underlying resource. Like
/// the other targets this is pure data; the GPU texture itself lives in the
/// adapter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureTarget {
    /// The texture size in physical pixels.
    pub size: Size,
    /// The texture's pixel format.
    pub format: PixelFormat,
    /// Whether the rendered result will be sampled as a texture afterward (a
    /// hint to the backend).
    pub sampled: bool,
}

impl TextureTarget {
    /// Creates a texture target of the given `size`, defaulting to
    /// [`PixelFormat::Rgba8Unorm`] and `sampled` set to `true`.
    #[must_use]
    pub const fn new(size: Size) -> Self {
        Self {
            size,
            format: PixelFormat::Rgba8Unorm,
            sampled: true,
        }
    }

    /// Sets the pixel format, returning the updated target.
    #[must_use]
    pub const fn with_format(mut self, format: PixelFormat) -> Self {
        self.format = format;
        self
    }

    /// Sets whether the result will be sampled afterward, returning the updated
    /// target.
    #[must_use]
    pub const fn sampled(mut self, sampled: bool) -> Self {
        self.sampled = sampled;
        self
    }

    /// The texture size in physical pixels.
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    /// The texture's pixel format.
    #[must_use]
    pub const fn format(&self) -> PixelFormat {
        self.format
    }

    /// Whether the result will be sampled as a texture afterward.
    #[must_use]
    pub const fn is_sampled(&self) -> bool {
        self.sampled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_defaults_to_rgba8_sampled() {
        let target = TextureTarget::new(Size::new(256.0, 256.0));
        assert_eq!(target.size(), Size::new(256.0, 256.0));
        assert_eq!(target.format(), PixelFormat::Rgba8Unorm);
        assert!(target.is_sampled());
    }

    #[test]
    fn builders_override_fields() {
        let target = TextureTarget::new(Size::new(1.0, 1.0))
            .with_format(PixelFormat::Bgra8Unorm)
            .sampled(false);
        assert_eq!(target.format(), PixelFormat::Bgra8Unorm);
        assert!(!target.is_sampled());
    }
}
