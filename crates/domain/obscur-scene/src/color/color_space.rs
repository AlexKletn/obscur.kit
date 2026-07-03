//! The color space a [`Color`](super::Color)'s components are expressed in.

/// The color space a [`Color`](super::Color)'s components are expressed in
/// (spec §5.7).
///
/// Colors are authored and exchanged in `Srgb` (the external semantics), while
/// compositing, blending, blur and filters operate in `LinearSrgb` (the linear
/// working space). `DisplayP3` describes wide-gamut content; on displays that
/// cannot reproduce it, consumers fall back to a safe sRGB approximation.
///
/// This enum intentionally lists only the spaces needed by the current model.
/// HDR and additional spaces (e.g. Rec. 2020, `ACEScg`) are deferred to a later
/// step and will be added as new variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSpace {
    /// Standard sRGB with the usual transfer function. This is the default
    /// external, authored semantics for colors.
    Srgb,
    /// Linear-light sRGB primaries (no transfer function). The working space
    /// used for compositing, blending, blur and filters.
    LinearSrgb,
    /// Display P3 wide-gamut primaries. Consumers fall back to sRGB on hardware
    /// that cannot reproduce the wider gamut.
    DisplayP3,
}

impl ColorSpace {
    /// Returns whether components are stored in linear light (no transfer
    /// function). True only for [`ColorSpace::LinearSrgb`].
    #[must_use]
    pub const fn is_linear(self) -> bool {
        matches!(self, Self::LinearSrgb)
    }
}

impl Default for ColorSpace {
    /// The default color space is [`ColorSpace::Srgb`], the external authored
    /// semantics.
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
