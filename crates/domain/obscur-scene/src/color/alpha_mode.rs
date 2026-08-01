#![doc = include_str!("alpha_mode.doc/_module.md")]

#[doc = include_str!("alpha_mode.doc/AlphaMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlphaMode {
    #[doc = include_str!("alpha_mode.doc/Straight.md")]
    Straight,
    #[doc = include_str!("alpha_mode.doc/Premultiplied.md")]
    Premultiplied,
}

impl AlphaMode {
    #[doc = include_str!("alpha_mode.doc/is_premultiplied.md")]
    #[must_use]
    pub const fn is_premultiplied(self) -> bool {
        matches!(self, Self::Premultiplied)
    }
}

impl Default for AlphaMode {
    #[doc = include_str!("alpha_mode.doc/default.md")]
    fn default() -> Self {
        Self::Straight
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_straight() {
        assert_eq!(AlphaMode::default(), AlphaMode::Straight);
    }

    #[test]
    fn is_premultiplied() {
        assert!(AlphaMode::Premultiplied.is_premultiplied());
        assert!(!AlphaMode::Straight.is_premultiplied());
    }
}
