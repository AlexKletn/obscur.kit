//! `ForeignSurface`: descriptor for externally-produced content in the scene (§5.5).

use obscur_geometry::Size;

use super::foreign_surface_kind::ForeignSurfaceKind;
use super::foreign_surface_limits::ForeignSurfaceLimits;

/// A descriptor for externally-produced content integrated into the scene
/// (Obscur Kit 2 specs §5.5).
///
/// A foreign surface hosts content produced outside the Paint model — video,
/// web views, cameras, native controls, or external GPU surfaces. It is either:
///
/// - **texture-backed** (`texture_backed == true`): the content is composited as
///   a texture and therefore integrates seamlessly, supporting the full
///   transform / clip / opacity / blur pipeline; or
/// - a **native child surface/window** (`texture_backed == false`): the content
///   is a platform-managed surface with explicit restrictions.
///
/// The `limits` field records, explicitly, which scene operations the surface
/// permits. Per §5.5 these capabilities are never silently assumed. This type is
/// a pure data descriptor — it holds no live GPU or renderer handle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForeignSurface {
    /// The kind of external content the surface hosts.
    kind: ForeignSurfaceKind,
    /// The logical size of the surface in logical pixels.
    size: Size,
    /// Whether the surface is composited as a texture (`true`) or is a native
    /// child surface/window (`false`).
    texture_backed: bool,
    /// The explicit operations the surface permits.
    limits: ForeignSurfaceLimits,
}

impl ForeignSurface {
    /// Creates a texture-backed foreign surface with unrestricted limits.
    ///
    /// Texture-backed surfaces composite like ordinary scene content, so
    /// `texture_backed` is `true` and `limits` is
    /// [`ForeignSurfaceLimits::unrestricted`].
    #[must_use]
    pub const fn new(kind: ForeignSurfaceKind, size: Size) -> Self {
        Self {
            kind,
            size,
            texture_backed: true,
            limits: ForeignSurfaceLimits::unrestricted(),
        }
    }

    /// Creates a native child surface/window with native-child limits.
    ///
    /// The surface is not texture-backed (`texture_backed` is `false`) and
    /// carries [`ForeignSurfaceLimits::native_child`].
    #[must_use]
    pub const fn native_child(kind: ForeignSurfaceKind, size: Size) -> Self {
        Self {
            kind,
            size,
            texture_backed: false,
            limits: ForeignSurfaceLimits::native_child(),
        }
    }

    /// Returns a copy of this descriptor with `texture_backed` set.
    #[must_use]
    pub const fn with_texture_backed(mut self, texture_backed: bool) -> Self {
        self.texture_backed = texture_backed;
        self
    }

    /// Returns a copy of this descriptor with the given limits.
    #[must_use]
    pub const fn with_limits(mut self, limits: ForeignSurfaceLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Returns the kind of external content the surface hosts.
    #[must_use]
    pub const fn kind(self) -> ForeignSurfaceKind {
        self.kind
    }

    /// Returns the logical size of the surface.
    #[must_use]
    pub const fn size(self) -> Size {
        self.size
    }

    /// Returns the explicit limits the surface imposes.
    #[must_use]
    pub const fn limits(self) -> ForeignSurfaceLimits {
        self.limits
    }

    /// Returns whether the surface is composited as a texture.
    #[must_use]
    pub const fn is_texture_backed(self) -> bool {
        self.texture_backed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_texture_backed_and_unrestricted() {
        let surface = ForeignSurface::new(ForeignSurfaceKind::Video, Size::new(640.0, 360.0));
        assert_eq!(surface.kind(), ForeignSurfaceKind::Video);
        assert_eq!(surface.size(), Size::new(640.0, 360.0));
        assert!(surface.is_texture_backed());
        assert_eq!(surface.limits(), ForeignSurfaceLimits::unrestricted());
    }

    #[test]
    fn native_child_is_not_texture_backed() {
        let surface = ForeignSurface::native_child(ForeignSurfaceKind::WebView, Size::ZERO);
        assert!(!surface.is_texture_backed());
        assert_eq!(surface.limits(), ForeignSurfaceLimits::native_child());
    }

    #[test]
    fn builders_override_defaults() {
        let surface = ForeignSurface::new(ForeignSurfaceKind::Camera, Size::ZERO)
            .with_texture_backed(false)
            .with_limits(ForeignSurfaceLimits::native_child());
        assert!(!surface.is_texture_backed());
        assert_eq!(surface.limits(), ForeignSurfaceLimits::native_child());
    }

    #[test]
    fn is_copy_and_eq() {
        let a = ForeignSurface::new(ForeignSurfaceKind::ExternalGpu, Size::new(1.0, 1.0));
        let b = a;
        assert_eq!(a, b);
    }
}
