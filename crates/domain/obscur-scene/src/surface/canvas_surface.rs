#![doc = include_str!("canvas_surface.doc/_module.md")]

use obscur_geometry::Size;

#[doc = include_str!("canvas_surface.doc/CanvasSurface.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasSurface {
    #[doc = include_str!("canvas_surface.doc/size.md")]
    size: Size,
    #[doc = include_str!("canvas_surface.doc/content_version.md")]
    content_version: u64,
}

impl CanvasSurface {
    #[doc = include_str!("canvas_surface.doc/new.md")]
    #[must_use]
    pub const fn new(size: Size) -> Self {
        Self {
            size,
            content_version: 0,
        }
    }

    #[doc = include_str!("canvas_surface.doc/with_content_version.md")]
    #[must_use]
    pub const fn with_content_version(mut self, content_version: u64) -> Self {
        self.content_version = content_version;
        self
    }

    #[doc = include_str!("canvas_surface.doc/bump_version.md")]
    pub fn bump_version(&mut self) {
        self.content_version = self.content_version.saturating_add(1);
    }

    #[doc = include_str!("canvas_surface.doc/size_2.md")]
    #[must_use]
    pub const fn size(self) -> Size {
        self.size
    }

    #[doc = include_str!("canvas_surface.doc/content_version_2.md")]
    #[must_use]
    pub const fn content_version(self) -> u64 {
        self.content_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_version_zero() {
        let surface = CanvasSurface::new(Size::new(100.0, 50.0));
        assert_eq!(surface.size(), Size::new(100.0, 50.0));
        assert_eq!(surface.content_version(), 0);
    }

    #[test]
    fn with_content_version_sets_version() {
        let surface = CanvasSurface::new(Size::ZERO).with_content_version(7);
        assert_eq!(surface.content_version(), 7);
    }

    #[test]
    fn bump_version_increments() {
        let mut surface = CanvasSurface::new(Size::ZERO);
        surface.bump_version();
        surface.bump_version();
        assert_eq!(surface.content_version(), 2);
    }

    #[test]
    fn bump_version_saturates() {
        let mut surface = CanvasSurface::new(Size::ZERO).with_content_version(u64::MAX);
        surface.bump_version();
        assert_eq!(surface.content_version(), u64::MAX);
    }

    #[test]
    fn is_copy_and_eq() {
        let a = CanvasSurface::new(Size::new(1.0, 1.0));
        let b = a;
        assert_eq!(a, b);
    }
}
