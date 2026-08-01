#![doc = include_str!("foreign_surface_limits.doc/_module.md")]

#[doc = include_str!("foreign_surface_limits.doc/ForeignSurfaceLimits.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ForeignSurfaceLimits {
    #[doc = include_str!("foreign_surface_limits.doc/allows_transform.md")]
    allows_transform: bool,
    #[doc = include_str!("foreign_surface_limits.doc/allows_clip.md")]
    allows_clip: bool,
    #[doc = include_str!("foreign_surface_limits.doc/allows_opacity.md")]
    allows_opacity: bool,
    #[doc = include_str!("foreign_surface_limits.doc/allows_blur.md")]
    allows_blur: bool,
}

impl ForeignSurfaceLimits {
    #[doc = include_str!("foreign_surface_limits.doc/unrestricted.md")]
    #[must_use]
    pub const fn unrestricted() -> Self {
        Self {
            allows_transform: true,
            allows_clip: true,
            allows_opacity: true,
            allows_blur: true,
        }
    }

    #[doc = include_str!("foreign_surface_limits.doc/native_child.md")]
    #[must_use]
    pub const fn native_child() -> Self {
        Self {
            allows_transform: true,
            allows_clip: true,
            allows_opacity: false,
            allows_blur: false,
        }
    }

    #[doc = include_str!("foreign_surface_limits.doc/allows_transform_2.md")]
    #[must_use]
    pub const fn allows_transform(self) -> bool {
        self.allows_transform
    }

    #[doc = include_str!("foreign_surface_limits.doc/allows_clip_2.md")]
    #[must_use]
    pub const fn allows_clip(self) -> bool {
        self.allows_clip
    }

    #[doc = include_str!("foreign_surface_limits.doc/allows_opacity_2.md")]
    #[must_use]
    pub const fn allows_opacity(self) -> bool {
        self.allows_opacity
    }

    #[doc = include_str!("foreign_surface_limits.doc/allows_blur_2.md")]
    #[must_use]
    pub const fn allows_blur(self) -> bool {
        self.allows_blur
    }
}

impl Default for ForeignSurfaceLimits {
    #[doc = include_str!("foreign_surface_limits.doc/default.md")]
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
