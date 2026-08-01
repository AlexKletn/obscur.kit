#![doc = include_str!("glyph_run.doc/_module.md")]

use super::PositionedGlyph;

#[doc = include_str!("glyph_run.doc/GlyphRun.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GlyphRun {
    #[doc = include_str!("glyph_run.doc/glyphs.md")]
    glyphs: Vec<PositionedGlyph>,
    #[doc = include_str!("glyph_run.doc/font.md")]
    font: u32,
    #[doc = include_str!("glyph_run.doc/font_size.md")]
    font_size: f32,
    #[doc = include_str!("glyph_run.doc/text.md")]
    text: Box<str>,
    #[doc = include_str!("glyph_run.doc/synth_oblique.md")]
    synth_oblique: bool,
    #[doc = include_str!("glyph_run.doc/synth_bold.md")]
    synth_bold: bool,
}

impl GlyphRun {
    #[doc = include_str!("glyph_run.doc/new.md")]
    #[must_use]
    pub fn new(glyphs: Vec<PositionedGlyph>, font_size: f32, text: impl Into<Box<str>>) -> Self {
        Self {
            glyphs,
            font: 0,
            font_size,
            text: text.into(),
            synth_oblique: false,
            synth_bold: false,
        }
    }

    #[doc = include_str!("glyph_run.doc/with_font.md")]
    #[must_use]
    pub const fn with_font(mut self, font: u32) -> Self {
        self.font = font;
        self
    }

    #[doc = include_str!("glyph_run.doc/with_synthesis.md")]
    #[must_use]
    pub const fn with_synthesis(mut self, oblique: bool, bold: bool) -> Self {
        self.synth_oblique = oblique;
        self.synth_bold = bold;
        self
    }

    #[doc = include_str!("glyph_run.doc/synth_oblique_2.md")]
    #[must_use]
    pub const fn synth_oblique(&self) -> bool {
        self.synth_oblique
    }

    #[doc = include_str!("glyph_run.doc/synth_bold_2.md")]
    #[must_use]
    pub const fn synth_bold(&self) -> bool {
        self.synth_bold
    }

    #[doc = include_str!("glyph_run.doc/font_2.md")]
    #[must_use]
    pub const fn font(&self) -> u32 {
        self.font
    }

    #[doc = include_str!("glyph_run.doc/glyphs_2.md")]
    #[must_use]
    pub fn glyphs(&self) -> &[PositionedGlyph] {
        &self.glyphs
    }

    #[doc = include_str!("glyph_run.doc/font_size_2.md")]
    #[must_use]
    pub const fn font_size(&self) -> f32 {
        self.font_size
    }

    #[doc = include_str!("glyph_run.doc/text_2.md")]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[doc = include_str!("glyph_run.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    #[doc = include_str!("glyph_run.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }

    #[doc = include_str!("glyph_run.doc/advance.md")]
    #[must_use]
    pub fn advance(&self) -> f32 {
        self.glyphs.iter().map(PositionedGlyph::advance).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_run() -> GlyphRun {
        GlyphRun::new(
            vec![
                PositionedGlyph::new(u32::from('a'), 0, 0.0, 16.0, 6.0),
                PositionedGlyph::new(u32::from('b'), 1, 6.0, 16.0, 6.0),
                PositionedGlyph::new(u32::from('c'), 2, 12.0, 16.0, 6.0),
            ],
            16.0,
            "abc",
        )
    }

    #[test]
    fn accessors_and_advance() {
        let run = sample_run();
        assert_eq!(run.len(), 3);
        assert!(!run.is_empty());
        assert_eq!(run.font_size(), 16.0);
        assert_eq!(run.text(), "abc");
        assert_eq!(run.advance(), 18.0);
        assert_eq!(run.glyphs()[1].glyph_id(), u32::from('b'));
        // A run built with `new` defaults to the primary face (font 0).
        assert_eq!(run.font(), 0);
    }

    #[test]
    fn with_font_tags_the_face_handle() {
        let run = sample_run().with_font(2);
        assert_eq!(run.font(), 2);
        // `with_font` only changes the font handle; glyphs and metrics are intact.
        assert_eq!(run.len(), 3);
        assert_eq!(run.font_size(), 16.0);
        // The handle participates in equality: same glyphs, different font differ.
        assert_ne!(run, sample_run());
        assert_eq!(run, sample_run().with_font(2));
    }

    #[test]
    fn empty_run() {
        let run = GlyphRun::new(Vec::new(), 16.0, "");
        assert!(run.is_empty());
        assert_eq!(run.len(), 0);
        assert_eq!(run.advance(), 0.0);
        assert_eq!(run.font(), 0);
    }

    #[test]
    fn synthesis_defaults_false_and_with_synthesis_sets_it() {
        let plain = sample_run();
        assert!(!plain.synth_oblique());
        assert!(!plain.synth_bold());

        let synthesized = sample_run().with_synthesis(true, true);
        assert!(synthesized.synth_oblique());
        assert!(synthesized.synth_bold());
        // The flags participate in equality (additive fields on the derive).
        assert_ne!(synthesized, sample_run());
        // Synthesis composes with the font handle; each flag is independent.
        let oblique_only = sample_run().with_font(2).with_synthesis(true, false);
        assert_eq!(oblique_only.font(), 2);
        assert!(oblique_only.synth_oblique());
        assert!(!oblique_only.synth_bold());
    }
}
