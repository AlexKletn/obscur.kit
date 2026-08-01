#![doc = include_str!("offscreen_surface.doc/_module.md")]

use obscur_geometry::{ScaleFactor, Size};

#[doc = include_str!("offscreen_surface.doc/OffscreenSurface.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OffscreenSurface {
    #[doc = include_str!("offscreen_surface.doc/size.md")]
    size: Size,
    #[doc = include_str!("offscreen_surface.doc/scale.md")]
    scale: ScaleFactor,
}

impl OffscreenSurface {
    #[doc = include_str!("offscreen_surface.doc/new.md")]
    #[must_use]
    pub const fn new(size: Size, scale: ScaleFactor) -> Self {
        Self { size, scale }
    }

    #[doc = include_str!("offscreen_surface.doc/at_scale.md")]
    #[must_use]
    pub const fn at_scale(size: Size, scale: ScaleFactor) -> Self {
        Self::new(size, scale)
    }

    #[doc = include_str!("offscreen_surface.doc/size_2.md")]
    #[must_use]
    pub const fn size(self) -> Size {
        self.size
    }

    #[doc = include_str!("offscreen_surface.doc/scale_2.md")]
    #[must_use]
    pub const fn scale(self) -> ScaleFactor {
        self.scale
    }

    #[doc = include_str!("offscreen_surface.doc/physical_size.md")]
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
