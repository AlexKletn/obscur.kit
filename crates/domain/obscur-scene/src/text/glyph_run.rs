//! A shaped, positioned run of glyphs carried by a `DrawText` command.

use super::PositionedGlyph;

#[doc = include_str!("../../docs/api/text/glyph_run.doc/GlyphRun.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GlyphRun {
    /// The positioned glyphs, in pen order.
    glyphs: Vec<PositionedGlyph>,
    /// The shaped run advance when some advances have no corresponding painted glyph.
    advance_override: Option<f32>,
    /// The font handle the glyph indices belong to (a bare face index; `0` is the
    /// primary face).
    font: u32,
    /// The font size the run was shaped at, in logical pixels.
    font_size: f32,
    /// The run's source text, retained as metadata.
    text: Box<str>,
    /// Whether the renderer should shear the glyphs to synthesize a slant.
    synth_oblique: bool,
    /// Whether the renderer should thicken the glyphs to synthesize a bold weight.
    synth_bold: bool,
}

impl GlyphRun {
    #[doc = include_str!("../../docs/api/text/glyph_run.doc/new.md")]
    #[must_use]
    pub fn new(glyphs: Vec<PositionedGlyph>, font_size: f32, text: impl Into<Box<str>>) -> Self {
        Self {
            glyphs,
            advance_override: None,
            font: 0,
            font_size,
            text: text.into(),
            synth_oblique: false,
            synth_bold: false,
        }
    }

    /// Sets the total logical advance independently of the glyphs that paint.
    #[must_use]
    pub const fn with_advance(mut self, advance: f32) -> Self {
        self.advance_override = Some(advance);
        self
    }

    /// Returns this run tagged with the `font` handle its glyph indices belong to
    /// (a bare face index; `0` is the primary face).
    #[must_use]
    pub const fn with_font(mut self, font: u32) -> Self {
        self.font = font;
        self
    }

    /// Returns this run flagged for synthetic styling: `oblique` shears the glyphs
    /// to fake a slant, `bold` thickens them to fake weight. Both default to
    /// `false` (the resolved face carries the requested style).
    #[must_use]
    pub const fn with_synthesis(mut self, oblique: bool, bold: bool) -> Self {
        self.synth_oblique = oblique;
        self.synth_bold = bold;
        self
    }

    /// Returns whether the renderer should shear the glyphs to synthesize a slant.
    #[must_use]
    pub const fn synth_oblique(&self) -> bool {
        self.synth_oblique
    }

    /// Returns whether the renderer should thicken the glyphs to synthesize a bold
    /// weight.
    #[must_use]
    pub const fn synth_bold(&self) -> bool {
        self.synth_bold
    }

    /// Returns the font handle the run's glyph indices belong to (a bare face
    /// index; `0` is the primary face).
    #[must_use]
    pub const fn font(&self) -> u32 {
        self.font
    }

    /// Returns the positioned glyphs, in pen order.
    #[must_use]
    pub fn glyphs(&self) -> &[PositionedGlyph] {
        &self.glyphs
    }

    /// Returns the font size the run was shaped at, in logical pixels.
    #[must_use]
    pub const fn font_size(&self) -> f32 {
        self.font_size
    }

    /// Returns the run's source text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the number of glyphs in the run.
    #[must_use]
    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    /// Returns whether the run contains no glyphs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }

    /// Returns the run’s total logical advance. By default it sums the glyph advances. A shaped run can supply an explicit advance for invisible spacing, such as tabs.
    #[must_use]
    pub fn advance(&self) -> f32 {
        self.advance_override
            .unwrap_or_else(|| self.glyphs.iter().map(PositionedGlyph::advance).sum())
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
        let with_spacing = run.clone().with_advance(24.0);
        assert_eq!(with_spacing.advance(), 24.0);
        assert_eq!(with_spacing.glyphs(), run.glyphs());
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
