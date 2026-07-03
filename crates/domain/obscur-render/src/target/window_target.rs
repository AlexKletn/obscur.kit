//! A swapchain-backed on-screen render surface (spec §5.2).

use obscur_geometry::Size;

use crate::target::PixelFormat;

/// The domain descriptor of an on-screen, swapchain-backed window surface
/// (spec §5.2).
///
/// This is a pure, backend-agnostic descriptor: the actual OS surface handle and
/// swapchain are held by the adapter that consumes it. It records the surface's
/// physical size, the scale factor mapping logical to physical pixels, and the
/// pixel format the swapchain presents in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowTarget {
    /// The surface size in physical pixels.
    pub size: Size,
    /// The scale factor mapping logical to physical pixels.
    pub scale_factor: f32,
    /// The pixel format the swapchain presents in.
    pub format: PixelFormat,
}

impl WindowTarget {
    /// Creates a window target of the given physical `size`, with an identity
    /// scale factor of `1.0` and the common window format
    /// [`PixelFormat::Bgra8Unorm`].
    #[must_use]
    pub const fn new(size: Size) -> Self {
        Self {
            size,
            scale_factor: 1.0,
            format: PixelFormat::Bgra8Unorm,
        }
    }

    /// Sets the scale factor mapping logical to physical pixels, returning the
    /// updated target.
    #[must_use]
    pub const fn with_scale_factor(mut self, scale_factor: f32) -> Self {
        self.scale_factor = scale_factor;
        self
    }

    /// Sets the swapchain pixel format, returning the updated target.
    #[must_use]
    pub const fn with_format(mut self, format: PixelFormat) -> Self {
        self.format = format;
        self
    }

    /// The surface size in physical pixels.
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    /// The scale factor mapping logical to physical pixels.
    #[must_use]
    pub const fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// The swapchain pixel format.
    #[must_use]
    pub const fn format(&self) -> PixelFormat {
        self.format
    }

    /// The surface size in logical pixels, i.e. the physical size divided by the
    /// scale factor.
    #[must_use]
    pub fn logical_size(&self) -> Size {
        Size::new(
            self.size.width / self.scale_factor,
            self.size.height / self.scale_factor,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_defaults_to_identity_scale_and_bgra() {
        let target = WindowTarget::new(Size::new(800.0, 600.0));
        assert_eq!(target.size(), Size::new(800.0, 600.0));
        assert_eq!(target.scale_factor(), 1.0);
        assert_eq!(target.format(), PixelFormat::Bgra8Unorm);
    }

    #[test]
    fn builders_override_fields() {
        let target = WindowTarget::new(Size::new(100.0, 100.0))
            .with_scale_factor(2.0)
            .with_format(PixelFormat::Rgba16Float);
        assert_eq!(target.scale_factor(), 2.0);
        assert_eq!(target.format(), PixelFormat::Rgba16Float);
    }

    #[test]
    fn logical_size_divides_by_scale_factor() {
        let target = WindowTarget::new(Size::new(1920.0, 1080.0)).with_scale_factor(2.0);
        assert_eq!(target.logical_size(), Size::new(960.0, 540.0));

        // Identity scale leaves logical == physical.
        let identity = WindowTarget::new(Size::new(640.0, 480.0));
        assert_eq!(identity.logical_size(), Size::new(640.0, 480.0));
    }
}
