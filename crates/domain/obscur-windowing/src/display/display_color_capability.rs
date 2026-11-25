//! A display's color and HDR capability (spec §6.6).

#[doc = include_str!("../../docs/api/display/display_color_capability.doc/DisplayColorCapability.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayColorCapability {
    /// The bit depth per color channel, or `None` if unknown.
    pub bit_depth: Option<u8>,
    /// Whether the display supports high dynamic range.
    pub hdr: bool,
    /// Whether the display covers a wide color gamut.
    pub wide_gamut: bool,
}

impl DisplayColorCapability {
    /// Creates the conservative SDR baseline: unknown bit depth, no HDR, no wide
    /// gamut.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bit_depth: None,
            hdr: false,
            wide_gamut: false,
        }
    }

    /// Sets the per-channel bit depth, returning the updated capability.
    #[must_use]
    pub const fn with_bit_depth(mut self, bit_depth: u8) -> Self {
        self.bit_depth = Some(bit_depth);
        self
    }

    /// Sets whether the display supports HDR, returning the updated capability.
    #[must_use]
    pub const fn with_hdr(mut self, hdr: bool) -> Self {
        self.hdr = hdr;
        self
    }

    /// Sets whether the display covers a wide color gamut, returning the updated
    /// capability.
    #[must_use]
    pub const fn with_wide_gamut(mut self, wide_gamut: bool) -> Self {
        self.wide_gamut = wide_gamut;
        self
    }

    /// Returns the per-channel bit depth, or `None` if unknown.
    #[must_use]
    pub const fn bit_depth(&self) -> Option<u8> {
        self.bit_depth
    }

    /// Returns whether the display supports high dynamic range.
    #[must_use]
    pub const fn is_hdr(&self) -> bool {
        self.hdr
    }

    /// Returns whether the display covers a wide color gamut.
    #[must_use]
    pub const fn is_wide_gamut(&self) -> bool {
        self.wide_gamut
    }
}

impl Default for DisplayColorCapability {
    /// The conservative SDR baseline, equal to [`DisplayColorCapability::new`].
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
