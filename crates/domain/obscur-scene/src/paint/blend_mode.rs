#![doc = include_str!("blend_mode.doc/_module.md")]

#[doc = include_str!("blend_mode.doc/BlendMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendMode {
    #[doc = include_str!("blend_mode.doc/Normal.md")]
    Normal,
    #[doc = include_str!("blend_mode.doc/Multiply.md")]
    Multiply,
    #[doc = include_str!("blend_mode.doc/Screen.md")]
    Screen,
    #[doc = include_str!("blend_mode.doc/Overlay.md")]
    Overlay,
    #[doc = include_str!("blend_mode.doc/Darken.md")]
    Darken,
    #[doc = include_str!("blend_mode.doc/Lighten.md")]
    Lighten,
    #[doc = include_str!("blend_mode.doc/ColorDodge.md")]
    ColorDodge,
    #[doc = include_str!("blend_mode.doc/ColorBurn.md")]
    ColorBurn,
    #[doc = include_str!("blend_mode.doc/HardLight.md")]
    HardLight,
    #[doc = include_str!("blend_mode.doc/SoftLight.md")]
    SoftLight,
    #[doc = include_str!("blend_mode.doc/Difference.md")]
    Difference,
    #[doc = include_str!("blend_mode.doc/Exclusion.md")]
    Exclusion,
}

impl BlendMode {
    #[doc = include_str!("blend_mode.doc/is_normal.md")]
    #[must_use]
    pub const fn is_normal(self) -> bool {
        matches!(self, Self::Normal)
    }
}

impl Default for BlendMode {
    #[doc = include_str!("blend_mode.doc/default.md")]
    fn default() -> Self {
        Self::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_normal() {
        assert_eq!(BlendMode::default(), BlendMode::Normal);
        assert!(BlendMode::default().is_normal());
    }

    #[test]
    fn non_normal_is_not_normal() {
        assert!(!BlendMode::Multiply.is_normal());
    }
}
