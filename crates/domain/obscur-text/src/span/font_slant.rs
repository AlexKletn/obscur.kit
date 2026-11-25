/// The slant (posture) of a font (spec §5.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontSlant {
    /// Upright glyphs.
    Normal,
    /// A true italic design (distinct glyph shapes).
    Italic,
    /// A slanted (obliqued) rendering of the upright design.
    Oblique,
}

impl Default for FontSlant {
    /// The default slant is [`FontSlant::Normal`].
    fn default() -> Self {
        Self::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_slant_default() {
        assert_eq!(FontSlant::default(), FontSlant::Normal);
    }
}
