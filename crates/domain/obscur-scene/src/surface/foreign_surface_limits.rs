//! `ForeignSurfaceLimits`: explicit restrictions a foreign surface may impose (§5.5).

#[doc = include_str!("../../docs/api/surface/foreign_surface_limits.doc/ForeignSurfaceLimits.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ForeignSurfaceLimits {
    /// Whether the surface may be transformed (translate/scale/rotate/etc.).
    allows_transform: bool,
    /// Whether the surface may be clipped by ancestor clips.
    allows_clip: bool,
    /// Whether the surface may be composited with reduced opacity.
    allows_opacity: bool,
    /// Whether a blur effect may be applied to the surface.
    allows_blur: bool,
}

impl ForeignSurfaceLimits {
    #[doc = include_str!("../../docs/api/surface/foreign_surface_limits.doc/unrestricted.md")]
    #[must_use]
    pub const fn unrestricted() -> Self {
        Self {
            allows_transform: true,
            allows_clip: true,
            allows_opacity: true,
            allows_blur: true,
        }
    }

    #[doc = include_str!("../../docs/api/surface/foreign_surface_limits.doc/native_child.md")]
    #[must_use]
    pub const fn native_child() -> Self {
        Self {
            allows_transform: true,
            allows_clip: true,
            allows_opacity: false,
            allows_blur: false,
        }
    }

    /// Returns whether the surface may be transformed.
    #[must_use]
    pub const fn allows_transform(self) -> bool {
        self.allows_transform
    }

    /// Returns whether the surface may be clipped.
    #[must_use]
    pub const fn allows_clip(self) -> bool {
        self.allows_clip
    }

    /// Returns whether the surface may be composited with reduced opacity.
    #[must_use]
    pub const fn allows_opacity(self) -> bool {
        self.allows_opacity
    }

    /// Returns whether a blur effect may be applied to the surface.
    #[must_use]
    pub const fn allows_blur(self) -> bool {
        self.allows_blur
    }
}

impl Default for ForeignSurfaceLimits {
    /// Defaults to [`unrestricted`](Self::unrestricted).
    fn default() -> Self {
        Self::unrestricted()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrestricted_allows_everything() {
        let limits = ForeignSurfaceLimits::unrestricted();
        assert!(limits.allows_transform());
        assert!(limits.allows_clip());
        assert!(limits.allows_opacity());
        assert!(limits.allows_blur());
    }

    #[test]
    fn native_child_restricts_opacity_and_blur() {
        let limits = ForeignSurfaceLimits::native_child();
        assert!(limits.allows_transform());
        assert!(limits.allows_clip());
        assert!(!limits.allows_opacity());
        assert!(!limits.allows_blur());
    }

    #[test]
    fn default_is_unrestricted() {
        assert_eq!(
            ForeignSurfaceLimits::default(),
            ForeignSurfaceLimits::unrestricted()
        );
    }
}
