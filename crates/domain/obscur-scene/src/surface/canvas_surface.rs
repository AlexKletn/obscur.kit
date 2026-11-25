//! `CanvasSurface`: descriptor for a built-in Canvas rendering surface (§5.3).

use obscur_geometry::Size;

#[doc = include_str!("../../docs/api/surface/canvas_surface.doc/CanvasSurface.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasSurface {
    /// The logical size of the Canvas surface in logical pixels.
    size: Size,
    /// A monotonic content version, incremented each time the Canvas redraws.
    content_version: u64,
}

impl CanvasSurface {
    /// Creates a Canvas surface of the given logical size with content version `0`.
    #[must_use]
    pub const fn new(size: Size) -> Self {
        Self {
            size,
            content_version: 0,
        }
    }

    /// Returns a copy of this descriptor with the given content version.
    #[must_use]
    pub const fn with_content_version(mut self, content_version: u64) -> Self {
        self.content_version = content_version;
        self
    }

    #[doc = include_str!("../../docs/api/surface/canvas_surface.doc/bump_version.md")]
    pub fn bump_version(&mut self) {
        self.content_version = self.content_version.saturating_add(1);
    }

    /// Returns the logical size of the Canvas surface.
    #[must_use]
    pub const fn size(self) -> Size {
        self.size
    }

    /// Returns the current content version.
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
