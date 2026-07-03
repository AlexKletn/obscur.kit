//! The selectable attributes used to pick a concrete font face (spec §5.18).

use crate::span::{FontSlant, FontWeight};

/// The selectable attributes a [`FontProvider`](crate::font::FontProvider) uses
/// to pick a concrete face within a family (spec §5.18).
///
/// A family names *which* typeface; these attributes name *which cut* of it —
/// its weight, slant, and width. The provider chooses the registered face whose
/// attributes are nearest the requested ones by [`FontAttributes::distance`].
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
    /// The penalty added to a [`distance`](Self::distance) when two faces differ
    /// in slant.
    ///
    /// It is large relative to a typical weight step (100) so a slant mismatch
    /// outweighs a modest weight difference: an upright request prefers a
    /// heavier upright face over a same-weight italic.
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

    /// Returns a heuristic distance from `self` to `other`, where `0` means an
    /// exact match and larger values mean a poorer fit (spec §5.18).
    ///
    /// A provider uses this to pick the nearest available face for a request.
    /// The distance is the weighted sum of three terms:
    ///
    /// * the absolute weight difference (`|weight - other.weight|`, one unit per
    ///   point on the `1..=1000` axis),
    /// * a flat [`SLANT_PENALTY`](Self::SLANT_PENALTY) when the slants differ,
    ///   and
    /// * the absolute difference of the two [`FontStretch::percentage`] values
    ///   (one unit per whole percent of width).
    ///
    /// It is symmetric and monotonic in the weight difference, so a request for
    /// a light weight ranks progressively heavier faces progressively worse.
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

/// A font's width, from [`UltraCondensed`](FontStretch::UltraCondensed) to
/// [`UltraExpanded`](FontStretch::UltraExpanded) (spec §5.18).
///
/// Variants are declared narrowest-to-widest, so the derived [`Ord`] ranks them
/// by width and providers can find the nearest available width. The default is
/// [`FontStretch::Normal`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontStretch {
    /// 50% of normal width.
    UltraCondensed,
    /// 62.5% of normal width.
    ExtraCondensed,
    /// 75% of normal width.
    Condensed,
    /// 87.5% of normal width.
    SemiCondensed,
    /// 100% (normal) width; the default.
    Normal,
    /// 112.5% of normal width.
    SemiExpanded,
    /// 125% of normal width.
    Expanded,
    /// 150% of normal width.
    ExtraExpanded,
    /// 200% of normal width.
    UltraExpanded,
}

impl FontStretch {
    /// Returns the CSS width percentage of this stretch, where
    /// [`FontStretch::Normal`] is `100.0` (spec §5.18).
    #[must_use]
    pub const fn percentage(&self) -> f32 {
        match self {
            Self::UltraCondensed => 50.0,
            Self::ExtraCondensed => 62.5,
            Self::Condensed => 75.0,
            Self::SemiCondensed => 87.5,
            Self::Normal => 100.0,
            Self::SemiExpanded => 112.5,
            Self::Expanded => 125.0,
            Self::ExtraExpanded => 150.0,
            Self::UltraExpanded => 200.0,
        }
    }
}

impl Default for FontStretch {
    /// The default width is [`FontStretch::Normal`].
    fn default() -> Self {
        Self::Normal
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
    fn stretch_percentages() {
        assert_eq!(FontStretch::UltraCondensed.percentage(), 50.0);
        assert_eq!(FontStretch::Normal.percentage(), 100.0);
        assert_eq!(FontStretch::UltraExpanded.percentage(), 200.0);
        assert_eq!(FontStretch::default(), FontStretch::Normal);
        // Declared order ranks by width for nearest-match.
        assert!(FontStretch::Condensed < FontStretch::Normal);
        assert!(FontStretch::Normal < FontStretch::Expanded);
    }

    #[test]
    fn distance_penalizes_stretch_difference() {
        let normal = FontAttributes::new();
        let condensed = FontAttributes::new().with_stretch(FontStretch::Condensed);
        // |100 - 75| = 25.
        assert_eq!(normal.distance(&condensed), 25);
    }
}
