//! A shaped, positioned run of glyphs carried by a `DrawText` command.

use super::PositionedGlyph;

/// The shaped output of one text run: its positioned glyphs plus the font size
/// and source text they were produced from (spec §5.10, §5.2).
///
/// A `GlyphRun` is the payload of a
/// [`DrawText`](crate::command::PaintCommand::DrawText) command. Text is shaped
/// **once**, in the `obscur-text` domain, and the resulting positioned glyphs are
/// carried here so every renderer consumes an identical run — restoring the
/// cross-backend determinism boundary (spec §5.2). Glyphs are stored in pen
/// order; each carries a position relative to the command's origin.
///
/// Each glyph's [`glyph_id`](PositionedGlyph::glyph_id) is a **font glyph index**
/// (the same contract as the domain `ShapedGlyph`), so a renderer draws it
/// directly against the host font with no remap.
///
/// The run also carries a [`font`](Self::font) handle: a plain `u32` index that
/// names *which* font face the run's glyph indices belong to. It is a bare index
/// (id `0` = the primary/first-choice face) rather than a typed identity so that
/// `obscur-scene` stays independent of `obscur-text`. Font fallback shapes one
/// paragraph across several faces and emits one `GlyphRun` per contiguous
/// same-font segment, so this handle is what lets a renderer pick the right font
/// for each run (spec §5.10).
///
/// [`text`](Self::text) retains the source string as metadata (accessibility and
/// diagnostics). Because it embeds `f32` metrics the type derives `PartialEq` but
/// not `Eq`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GlyphRun {
    /// The positioned glyphs, in pen order.
    glyphs: Vec<PositionedGlyph>,
    /// The font handle the glyph indices belong to (a bare face index; `0` is the
    /// primary face).
    font: u32,
    /// The font size the run was shaped at, in logical pixels.
    font_size: f32,
    /// The run's source text, retained as metadata.
    text: Box<str>,
}

impl GlyphRun {
    /// Creates a run from its positioned `glyphs`, the `font_size` they were
    /// shaped at, and the source `text` they were produced from.
    ///
    /// The [`font`](Self::font) handle defaults to `0` (the primary face); tag a
    /// non-primary run with [`with_font`](Self::with_font).
    #[must_use]
    pub fn new(glyphs: Vec<PositionedGlyph>, font_size: f32, text: impl Into<Box<str>>) -> Self {
        Self {
            glyphs,
            font: 0,
            font_size,
            text: text.into(),
        }
    }

    /// Returns this run tagged with the `font` handle its glyph indices belong to
    /// (a bare face index; `0` is the primary face).
    #[must_use]
    pub const fn with_font(mut self, font: u32) -> Self {
        self.font = font;
        self
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

    /// Returns the run's total advance width: the sum of every glyph's
    /// [`advance`](PositionedGlyph::advance), in logical pixels.
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
}
