#![doc = include_str!("color_space.doc/_module.md")]

#[doc = include_str!("color_space.doc/ColorSpace.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSpace {
    #[doc = include_str!("color_space.doc/Srgb.md")]
    Srgb,
    #[doc = include_str!("color_space.doc/LinearSrgb.md")]
    LinearSrgb,
    #[doc = include_str!("color_space.doc/DisplayP3.md")]
    DisplayP3,
}

impl ColorSpace {
    #[doc = include_str!("color_space.doc/is_linear.md")]
    #[must_use]
    pub const fn is_linear(self) -> bool {
        matches!(self, Self::LinearSrgb)
    }
}

impl Default for ColorSpace {
    #[doc = include_str!("color_space.doc/default.md")]
    fn default() -> Self {
        Self::Srgb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_srgb() {
        assert_eq!(ColorSpace::default(), ColorSpace::Srgb);
    }

    #[test]
    fn only_linear_srgb_is_linear() {
        assert!(ColorSpace::LinearSrgb.is_linear());
        assert!(!ColorSpace::Srgb.is_linear());
        assert!(!ColorSpace::DisplayP3.is_linear());
    }
}
