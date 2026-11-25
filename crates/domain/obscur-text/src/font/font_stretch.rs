#[doc = include_str!("../../docs/api/font/font_attributes.doc/FontStretch.md")]
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
    fn stretch_percentages() {
        assert_eq!(FontStretch::UltraCondensed.percentage(), 50.0);
        assert_eq!(FontStretch::Normal.percentage(), 100.0);
        assert_eq!(FontStretch::UltraExpanded.percentage(), 200.0);
        assert_eq!(FontStretch::default(), FontStretch::Normal);
        // Declared order ranks by width for nearest-match.
        assert!(FontStretch::Condensed < FontStretch::Normal);
        assert!(FontStretch::Normal < FontStretch::Expanded);
    }
}
