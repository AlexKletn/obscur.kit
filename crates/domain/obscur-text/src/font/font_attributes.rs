//! The selectable attributes used to pick a concrete font face (spec §5.18).

use crate::span::{FontSlant, FontWeight};

use super::FontStretch;

#[doc = include_str!("../../docs/api/font/font_attributes.doc/FontAttributes.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FontAttributes {
    /// The requested weight (default [`FontWeight::NORMAL`]).
    pub weight: FontWeight,
    /// The requested slant (default [`FontSlant::Normal`]).
    pub slant: FontSlant,
    /// The requested width (default [`FontStretch::Normal`]).
    pub stretch: FontStretch,
}

impl FontAttributes {
    #[doc = include_str!("../../docs/api/font/font_attributes.doc/SLANT_PENALTY.md")]
    const SLANT_PENALTY: u32 = 500;

    /// Creates attributes at their defaults: [`FontWeight::NORMAL`],
    /// [`FontSlant::Normal`], and [`FontStretch::Normal`].
    #[must_use]
    pub const fn new() -> Self {
        Self {
            weight: FontWeight::NORMAL,
            slant: FontSlant::Normal,
            stretch: FontStretch::Normal,
        }
    }

    /// Returns these attributes with `weight` set.
    #[must_use]
    pub const fn with_weight(mut self, weight: FontWeight) -> Self {
        self.weight = weight;
        self
    }

    /// Returns these attributes with `slant` set.
    #[must_use]
    pub const fn with_slant(mut self, slant: FontSlant) -> Self {
        self.slant = slant;
        self
    }

    /// Returns these attributes with `stretch` set.
    #[must_use]
    pub const fn with_stretch(mut self, stretch: FontStretch) -> Self {
        self.stretch = stretch;
        self
    }

    #[doc = include_str!("../../docs/api/font/font_attributes.doc/distance.md")]
    #[must_use]
    pub fn distance(&self, other: &FontAttributes) -> u32 {
        let weight_diff = u32::from(self.weight.value().abs_diff(other.weight.value()));
        let slant_penalty = if self.slant == other.slant {
            0
        } else {
            Self::SLANT_PENALTY
        };
        let stretch_diff = (self.stretch.percentage() - other.stretch.percentage()).abs();
        // `stretch_diff` is a non-negative width difference in whole-ish
        // percent (at most 150), so rounding to `u32` neither truncates
        // meaningfully nor loses a sign.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let stretch_penalty = stretch_diff.round() as u32;
        weight_diff + slant_penalty + stretch_penalty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let attrs = FontAttributes::default();
        assert_eq!(attrs, FontAttributes::new());
        assert_eq!(attrs.weight, FontWeight::NORMAL);
        assert_eq!(attrs.slant, FontSlant::Normal);
        assert_eq!(attrs.stretch, FontStretch::Normal);
    }

    #[test]
    fn builders() {
        let attrs = FontAttributes::new()
            .with_weight(FontWeight::BOLD)
            .with_slant(FontSlant::Italic)
            .with_stretch(FontStretch::Condensed);
        assert_eq!(attrs.weight, FontWeight::BOLD);
        assert_eq!(attrs.slant, FontSlant::Italic);
        assert_eq!(attrs.stretch, FontStretch::Condensed);
    }

    #[test]
    fn distance_identical_is_zero() {
        let attrs = FontAttributes::new().with_weight(FontWeight::MEDIUM);
        assert_eq!(attrs.distance(&attrs), 0);
    }

    #[test]
    fn distance_grows_and_is_monotonic_in_weight() {
        let base = FontAttributes::new().with_weight(FontWeight::new(300));
        let near = FontAttributes::new().with_weight(FontWeight::new(400));
        let far = FontAttributes::new().with_weight(FontWeight::new(700));
        assert!(base.distance(&far) > base.distance(&near));
        assert!(base.distance(&near) > 0);
        // Symmetric.
        assert_eq!(base.distance(&far), far.distance(&base));
    }

    #[test]
    fn distance_penalizes_slant_mismatch() {
        let upright = FontAttributes::new();
        let italic = FontAttributes::new().with_slant(FontSlant::Italic);
        assert_eq!(upright.distance(&italic), FontAttributes::SLANT_PENALTY);
        // A same-weight italic is worse than a modestly heavier upright.
        let heavier = FontAttributes::new().with_weight(FontWeight::new(500));
        assert!(upright.distance(&italic) > upright.distance(&heavier));
    }

    #[test]
    fn distance_penalizes_stretch_difference() {
        let normal = FontAttributes::new();
        let condensed = FontAttributes::new().with_stretch(FontStretch::Condensed);
        // |100 - 75| = 25.
        assert_eq!(normal.distance(&condensed), 25);
    }
}
