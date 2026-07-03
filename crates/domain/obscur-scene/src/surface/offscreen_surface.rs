//! `OffscreenSurface`: descriptor for an offscreen render target (§5.6).

use obscur_geometry::{ScaleFactor, Size};

/// A descriptor for an offscreen render target (Obscur Kit 2 specs §5.6).
///
/// Offscreen output renders a subtree or scene to a texture, bitmap, thumbnail,
/// preview, or node screenshot. This is a pure data descriptor — Phase 1
/// captures the target `size` (in logical pixels) and the `scale` (DPI) to
/// render at. Additional inputs described by §5.6 — theme, locale, and
/// deterministic-resource configuration — are added in a later phase.
///
/// The physical pixel size of the target is [`Self::physical_size`], each axis
/// of `size` multiplied by `scale`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OffscreenSurface {
    /// The target size in logical pixels.
    size: Size,
    /// The DPI scale factor to render at.
    scale: ScaleFactor,
}

impl OffscreenSurface {
    /// Creates an offscreen render target of `size` logical pixels at `scale`.
    #[must_use]
    pub const fn new(size: Size, scale: ScaleFactor) -> Self {
        Self { size, scale }
    }

    /// Creates an offscreen render target of `size` logical pixels at `scale`.
    ///
    /// A readability alias for [`Self::new`] that reads naturally at call sites,
    /// e.g. `OffscreenSurface::at_scale(size, ScaleFactor::new(2.0))`.
    #[must_use]
    pub const fn at_scale(size: Size, scale: ScaleFactor) -> Self {
        Self::new(size, scale)
    }

    /// Returns the target size in logical pixels.
    #[must_use]
    pub const fn size(self) -> Size {
        self.size
    }

    /// Returns the DPI scale factor the target renders at.
    #[must_use]
    pub const fn scale(self) -> ScaleFactor {
        self.scale
    }

    /// Returns the physical pixel size of the target.
    ///
    /// Each axis of the logical [`Self::size`] is multiplied by the scale
    /// multiplier from [`Self::scale`].
    #[must_use]
    pub fn physical_size(self) -> Size {
        let factor = self.scale.get();
        Size::new(self.size.width * factor, self.size.height * factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_return_inputs() {
        let surface = OffscreenSurface::new(Size::new(320.0, 240.0), ScaleFactor::new(2.0));
        assert_eq!(surface.size(), Size::new(320.0, 240.0));
        assert_eq!(surface.scale(), ScaleFactor::new(2.0));
    }

    #[test]
    fn at_scale_matches_new() {
        let size = Size::new(10.0, 20.0);
        let scale = ScaleFactor::new(1.5);
        assert_eq!(
            OffscreenSurface::at_scale(size, scale),
            OffscreenSurface::new(size, scale)
        );
    }

    #[test]
    fn physical_size_multiplies_each_axis() {
        let surface = OffscreenSurface::new(Size::new(100.0, 50.0), ScaleFactor::new(2.0));
        assert_eq!(surface.physical_size(), Size::new(200.0, 100.0));
    }

    #[test]
    fn physical_size_at_identity_equals_logical() {
        let surface = OffscreenSurface::new(Size::new(64.0, 32.0), ScaleFactor::IDENTITY);
        assert_eq!(surface.physical_size(), Size::new(64.0, 32.0));
    }
}
