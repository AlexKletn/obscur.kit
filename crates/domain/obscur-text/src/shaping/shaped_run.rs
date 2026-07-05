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
#[derive(Debug, Clone, PartialEq)]
pub struct ShapedRun {
    /// The font the run's glyph indices belong to.
    font: FontId,
    /// The shaped glyphs and their run metrics.
    run: GlyphRun,
    /// The bidi embedding level of the source run (even = LTR, odd = RTL).
    level: u8,
}

impl ShapedRun {
    /// Creates a run tagged with the `font` its glyph indices belong to, at the
    /// default bidi embedding level `0` (LTR).
    ///
    /// Set the level with [`with_level`](Self::with_level) when it is known.
    #[must_use]
    pub const fn new(font: FontId, run: GlyphRun) -> Self {
        Self {
            font,
            run,
            level: 0,
        }
    }

    /// Returns this run tagged with the bidi embedding `level` of its source item
    /// (even = LTR, odd = RTL).
    #[must_use]
    pub const fn with_level(mut self, level: u8) -> Self {
        self.level = level;
        self
    }

    /// Returns the font the run's glyph indices belong to.
    #[must_use]
    pub const fn font(&self) -> FontId {
        self.font
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
    fn level_defaults_to_zero_and_with_level_sets_it() {
        let ltr = ShapedRun::new(FontId::new(3), sample_run());
        assert_eq!(ltr.level(), 0);
        let rtl = ShapedRun::new(FontId::new(3), sample_run()).with_level(1);
        assert_eq!(rtl.level(), 1);
        // The level participates in equality (additive field on the derive).
        assert_ne!(ltr, rtl);
    }
}
