//! A shaped run tagged with the font its glyphs belong to (spec §5.10).

use crate::font::FontId;
use crate::shaping::GlyphRun;

#[doc = include_str!("../../docs/api/shaping/shaped_run.doc/ShapedRun.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ShapedRun {
    /// The font the run's glyph indices belong to.
    font: FontId,
    /// The shaped glyphs and their run metrics.
    run: GlyphRun,
    /// The bidi embedding level of the source run (even = LTR, odd = RTL).
    level: u8,
    /// Whether the glyphs should be sheared to synthesize a slant the matched
    /// face lacks (spec §5.18).
    synth_oblique: bool,
    /// Whether the glyphs should be thickened to synthesize a bold weight the
    /// matched face lacks (spec §5.18).
    synth_bold: bool,
}

impl ShapedRun {
    #[doc = include_str!("../../docs/api/shaping/shaped_run.doc/new.md")]
    #[must_use]
    pub const fn new(font: FontId, run: GlyphRun) -> Self {
        Self {
            font,
            run,
            level: 0,
            synth_oblique: false,
            synth_bold: false,
        }
    }

    /// Returns this run tagged with the bidi embedding `level` of its source item
    /// (even = LTR, odd = RTL).
    #[must_use]
    pub const fn with_level(mut self, level: u8) -> Self {
        self.level = level;
        self
    }

    /// Returns this run flagged for synthetic styling: `oblique` shears the glyphs
    /// to fake a slant, `bold` thickens them to fake weight (spec §5.18).
    #[must_use]
    pub const fn with_synthesis(mut self, oblique: bool, bold: bool) -> Self {
        self.synth_oblique = oblique;
        self.synth_bold = bold;
        self
    }

    /// Returns the font the run's glyph indices belong to.
    #[must_use]
    pub const fn font(&self) -> FontId {
        self.font
    }

    /// Returns whether the glyphs should be sheared to synthesize a slant the
    /// matched face lacks (spec §5.18).
    #[must_use]
    pub const fn synth_oblique(&self) -> bool {
        self.synth_oblique
    }

    /// Returns whether the glyphs should be thickened to synthesize a bold weight
    /// the matched face lacks (spec §5.18).
    #[must_use]
    pub const fn synth_bold(&self) -> bool {
        self.synth_bold
    }

    /// Returns the bidi embedding level of the source run (even = LTR, odd = RTL).
    #[must_use]
    pub const fn level(&self) -> u8 {
        self.level
    }

    /// Returns the shaped glyph run.
    #[must_use]
    pub const fn run(&self) -> &GlyphRun {
        &self.run
    }

    /// Consumes the tag, returning the owned [`GlyphRun`].
    #[must_use]
    pub fn into_run(self) -> GlyphRun {
        self.run
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shaping::{ShapedGlyph, TextDirection};
    use crate::span::TextRange;

    fn sample_run() -> GlyphRun {
        GlyphRun::new(
            vec![ShapedGlyph::new(1, 0, 6.0), ShapedGlyph::new(2, 1, 6.0)],
            TextDirection::LeftToRight,
            10.0,
            TextRange::new(0, 2),
        )
    }

    #[test]
    fn holds_font_and_run() {
        let shaped = ShapedRun::new(FontId::new(3), sample_run());
        assert_eq!(shaped.font(), FontId::new(3));
        assert_eq!(shaped.run(), &sample_run());
    }

    #[test]
    fn into_run_yields_owned_run() {
        let shaped = ShapedRun::new(FontId::PRIMARY, sample_run());
        assert_eq!(shaped.into_run(), sample_run());
    }

    #[test]
    fn synthesis_defaults_false_and_with_synthesis_sets_it() {
        let plain = ShapedRun::new(FontId::new(3), sample_run());
        assert!(!plain.synth_oblique());
        assert!(!plain.synth_bold());

        let synthesized = ShapedRun::new(FontId::new(3), sample_run()).with_synthesis(true, true);
        assert!(synthesized.synth_oblique());
        assert!(synthesized.synth_bold());
        // The flags participate in equality (additive fields on the derive).
        assert_ne!(plain, synthesized);

        // Synthesis composes with level and each flag is independent.
        let oblique_only = ShapedRun::new(FontId::new(3), sample_run())
            .with_level(1)
            .with_synthesis(true, false);
        assert_eq!(oblique_only.level(), 1);
        assert!(oblique_only.synth_oblique());
        assert!(!oblique_only.synth_bold());
    }

    #[test]
    fn level_defaults_to_zero_and_with_level_sets_it() {
        let ltr = ShapedRun::new(FontId::new(3), sample_run());
        assert_eq!(ltr.level(), 0);
        let rtl = ShapedRun::new(FontId::new(3), sample_run()).with_level(1);
        assert_eq!(rtl.level(), 1);
        // The level participates in equality (additive field on the derive).
        assert_ne!(ltr, rtl);
    }
}
