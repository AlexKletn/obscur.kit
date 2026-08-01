#![doc = include_str!("shaped_run.doc/_module.md")]

use crate::font::FontId;
use crate::shaping::GlyphRun;

#[doc = include_str!("shaped_run.doc/ShapedRun.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ShapedRun {
    #[doc = include_str!("shaped_run.doc/font.md")]
    font: FontId,
    #[doc = include_str!("shaped_run.doc/run.md")]
    run: GlyphRun,
    #[doc = include_str!("shaped_run.doc/level.md")]
    level: u8,
    #[doc = include_str!("shaped_run.doc/synth_oblique.md")]
    synth_oblique: bool,
    #[doc = include_str!("shaped_run.doc/synth_bold.md")]
    synth_bold: bool,
}

impl ShapedRun {
    #[doc = include_str!("shaped_run.doc/new.md")]
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

    #[doc = include_str!("shaped_run.doc/with_level.md")]
    #[must_use]
    pub const fn with_level(mut self, level: u8) -> Self {
        self.level = level;
        self
    }

    #[doc = include_str!("shaped_run.doc/with_synthesis.md")]
    #[must_use]
    pub const fn with_synthesis(mut self, oblique: bool, bold: bool) -> Self {
        self.synth_oblique = oblique;
        self.synth_bold = bold;
        self
    }

    #[doc = include_str!("shaped_run.doc/font_2.md")]
    #[must_use]
    pub const fn font(&self) -> FontId {
        self.font
    }

    #[doc = include_str!("shaped_run.doc/synth_oblique_2.md")]
    #[must_use]
    pub const fn synth_oblique(&self) -> bool {
        self.synth_oblique
    }

    #[doc = include_str!("shaped_run.doc/synth_bold_2.md")]
    #[must_use]
    pub const fn synth_bold(&self) -> bool {
        self.synth_bold
    }

    #[doc = include_str!("shaped_run.doc/level_2.md")]
    #[must_use]
    pub const fn level(&self) -> u8 {
        self.level
    }

    #[doc = include_str!("shaped_run.doc/run_2.md")]
    #[must_use]
    pub const fn run(&self) -> &GlyphRun {
        &self.run
    }

    #[doc = include_str!("shaped_run.doc/into_run.md")]
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
