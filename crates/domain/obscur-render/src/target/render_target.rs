//! Where a frame renders: the umbrella of render destinations (spec §5.2, §5.6).

use obscur_geometry::Size;

use crate::target::{BitmapTarget, TextureTarget, WindowTarget};

/// The pixel format of a [`RenderTarget`]'s color buffer (spec §5.2).
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

/// A destination a [`RenderFrame`](crate::frame::RenderFrame) is rendered into:
/// an on-screen window, an offscreen GPU texture, or an offscreen CPU bitmap
/// (spec §5.2, §5.6).
///
/// This is the backend-agnostic descriptor of *where* a frame renders. The base
/// 2D backend (Vello) is hidden behind the renderer abstraction; this type
/// models the destination as pure data, holding no GPU or OS handles — those
/// live in the adapter that consumes this descriptor (spec §5.2). Offscreen
/// destinations (texture and bitmap) implement the offscreen-output surface of
/// spec §5.6.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderTarget {
    /// An on-screen, swapchain-backed window surface.
    Window(WindowTarget),
    /// An offscreen render-to-GPU-texture destination (spec §5.6).
    Texture(TextureTarget),
    /// An offscreen render-to-CPU-bitmap destination (spec §5.6).
    Bitmap(BitmapTarget),
}

impl RenderTarget {
    /// Wraps an on-screen [`WindowTarget`].
    #[must_use]
    pub const fn window(target: WindowTarget) -> Self {
        Self::Window(target)
    }

    /// Wraps an offscreen [`TextureTarget`] (spec §5.6).
    #[must_use]
    pub const fn texture(target: TextureTarget) -> Self {
        Self::Texture(target)
    }

    /// Wraps an offscreen [`BitmapTarget`] (spec §5.6).
    #[must_use]
    pub const fn bitmap(target: BitmapTarget) -> Self {
        Self::Bitmap(target)
    }

    /// The size of the destination in physical pixels, delegating to the inner
    /// target.
    #[must_use]
    pub const fn size(&self) -> Size {
        match self {
            Self::Window(target) => target.size(),
            Self::Texture(target) => target.size(),
            Self::Bitmap(target) => target.size(),
        }
    }

    /// Whether this target is an on-screen window.
    #[must_use]
    pub const fn is_window(&self) -> bool {
        matches!(self, Self::Window(_))
    }

    /// Whether this target is an offscreen GPU texture.
    #[must_use]
    pub const fn is_texture(&self) -> bool {
        matches!(self, Self::Texture(_))
    }

    /// Whether this target is an offscreen CPU bitmap.
    #[must_use]
    pub const fn is_bitmap(&self) -> bool {
        matches!(self, Self::Bitmap(_))
    }

    /// Whether this target renders offscreen rather than to a window.
    ///
    /// True for the texture and bitmap destinations, which implement the
    /// offscreen output of spec §5.6.
    #[must_use]
    pub const fn is_offscreen(&self) -> bool {
        matches!(self, Self::Texture(_) | Self::Bitmap(_))
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

    #[test]
    fn size_delegates_to_inner_target() {
        let window = RenderTarget::window(WindowTarget::new(Size::new(800.0, 600.0)));
        let texture = RenderTarget::texture(TextureTarget::new(Size::new(256.0, 128.0)));
        let bitmap = RenderTarget::bitmap(BitmapTarget::new(Size::new(64.0, 32.0)));
        assert_eq!(window.size(), Size::new(800.0, 600.0));
        assert_eq!(texture.size(), Size::new(256.0, 128.0));
        assert_eq!(bitmap.size(), Size::new(64.0, 32.0));
    }

    #[test]
    fn predicates_and_is_offscreen_across_variants() {
        let window = RenderTarget::window(WindowTarget::new(Size::new(1.0, 1.0)));
        let texture = RenderTarget::texture(TextureTarget::new(Size::new(1.0, 1.0)));
        let bitmap = RenderTarget::bitmap(BitmapTarget::new(Size::new(1.0, 1.0)));

        assert!(window.is_window() && !window.is_texture() && !window.is_bitmap());
        assert!(texture.is_texture() && !texture.is_window());
        assert!(bitmap.is_bitmap() && !bitmap.is_window());

        // §5.6: texture and bitmap are offscreen; a window is not.
        assert!(!window.is_offscreen());
        assert!(texture.is_offscreen());
        assert!(bitmap.is_offscreen());
    }
}
