#![doc = include_str!("extend_mode.doc/_module.md")]

#[doc = include_str!("extend_mode.doc/ExtendMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtendMode {
    #[doc = include_str!("extend_mode.doc/Pad.md")]
    Pad,
    #[doc = include_str!("extend_mode.doc/Repeat.md")]
    Repeat,
    #[doc = include_str!("extend_mode.doc/Reflect.md")]
    Reflect,
}

impl ExtendMode {
    #[doc = include_str!("extend_mode.doc/is_pad.md")]
    #[must_use]
    pub const fn is_pad(self) -> bool {
        matches!(self, Self::Pad)
    }
}

impl Default for ExtendMode {
    #[doc = include_str!("extend_mode.doc/default.md")]
    fn default() -> Self {
        Self::Pad
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_pad() {
        assert_eq!(ExtendMode::default(), ExtendMode::Pad);
        assert!(ExtendMode::default().is_pad());
    }

    #[test]
    fn variants_distinct() {
        assert_ne!(ExtendMode::Repeat, ExtendMode::Reflect);
    }
}
