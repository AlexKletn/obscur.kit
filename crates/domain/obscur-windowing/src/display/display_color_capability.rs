#![doc = include_str!("display_color_capability.doc/_module.md")]

#[doc = include_str!("display_color_capability.doc/DisplayColorCapability.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayColorCapability {
    #[doc = include_str!("display_color_capability.doc/bit_depth.md")]
    pub bit_depth: Option<u8>,
    #[doc = include_str!("display_color_capability.doc/hdr.md")]
    pub hdr: bool,
    #[doc = include_str!("display_color_capability.doc/wide_gamut.md")]
    pub wide_gamut: bool,
}

impl DisplayColorCapability {
    #[doc = include_str!("display_color_capability.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bit_depth: None,
            hdr: false,
            wide_gamut: false,
        }
    }

    #[doc = include_str!("display_color_capability.doc/with_bit_depth.md")]
    #[must_use]
    pub const fn with_bit_depth(mut self, bit_depth: u8) -> Self {
        self.bit_depth = Some(bit_depth);
        self
    }

    #[doc = include_str!("display_color_capability.doc/with_hdr.md")]
    #[must_use]
    pub const fn with_hdr(mut self, hdr: bool) -> Self {
        self.hdr = hdr;
        self
    }

    #[doc = include_str!("display_color_capability.doc/with_wide_gamut.md")]
    #[must_use]
    pub const fn with_wide_gamut(mut self, wide_gamut: bool) -> Self {
        self.wide_gamut = wide_gamut;
        self
    }

    #[doc = include_str!("display_color_capability.doc/bit_depth_2.md")]
    #[must_use]
    pub const fn bit_depth(&self) -> Option<u8> {
        self.bit_depth
    }

    #[doc = include_str!("display_color_capability.doc/is_hdr.md")]
    #[must_use]
    pub const fn is_hdr(&self) -> bool {
        self.hdr
    }

    #[doc = include_str!("display_color_capability.doc/is_wide_gamut.md")]
    #[must_use]
    pub const fn is_wide_gamut(&self) -> bool {
        self.wide_gamut
    }
}

impl Default for DisplayColorCapability {
    #[doc = include_str!("display_color_capability.doc/default.md")]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_sdr_baseline() {
        let c = DisplayColorCapability::default();
        assert_eq!(c.bit_depth(), None);
        assert!(!c.is_hdr());
        assert!(!c.is_wide_gamut());
    }

    #[test]
    fn builder_sets_fields_and_is_hdr() {
        let c = DisplayColorCapability::new()
            .with_bit_depth(10)
            .with_hdr(true)
            .with_wide_gamut(true);
        assert_eq!(c.bit_depth(), Some(10));
        assert!(c.is_hdr());
        assert!(c.is_wide_gamut());
    }
}
