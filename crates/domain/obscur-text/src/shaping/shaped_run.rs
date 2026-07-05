//! A shaped run tagged with the font its glyphs belong to (spec §5.10).

use crate::font::FontId;
use crate::shaping::GlyphRun;

/// A [`GlyphRun`] tagged with the [`FontId`] its glyph indices belong to
/// (spec §5.10).
///
/// A [`GlyphRun`]'s glyph ids are indices *into a specific font face*, but the
/// run itself does not record which face. Font fallback shapes one string with
/// several faces, so the result is a `Vec<ShapedRun>`: each element pairs a run
/// with the [`FontId`] its glyphs must be looked up and rasterized against. This
/// keeps the multi-font result expressible without changing [`GlyphRun`] itself.
///
/// The run also carries its source item's bidi embedding `level` (spec §5.10,
/// UAX#9): even levels are left-to-right, odd right-to-left. Line layout uses it
/// to reorder a line's segments into visual order (the L2 rule); `new` defaults
/// it to `0` (LTR) so callers that don't yet know the level keep compiling.
///
/// Because [`GlyphRun`] embeds `f32` metrics this type derives `PartialEq` but
/// not `Eq`.
/// The run also carries two per-run synthesis hints
/// ([`synth_oblique`](Self::synth_oblique), [`synth_bold`](Self::synth_bold)):
/// when the font resolver matched the run to an upright/lighter face because no
/// true italic/bold cut was registered, the renderer must shear/thicken the
/// glyphs to fake the requested style (spec §5.18). Both default to `false`;
/// clusters and glyphs are unchanged — synthesis is purely a render-time effect.
///
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
    /// Creates a run tagged with the `font` its glyph indices belong to, at the
    /// default bidi embedding level `0` (LTR) and with no synthesis.
    ///
    /// Set the level with [`with_level`](Self::with_level) when it is known, and
    /// the synthesis hints with [`with_synthesis`](Self::with_synthesis).
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
