use crate::shaping::ShapedGlyph;

/// A word: the glyphs between two adjacent break opportunities, its advance
/// width, and whether a mandatory break follows it.
pub(super) struct GlyphWord {
    pub(super) glyphs: Vec<ShapedGlyph>,
    pub(super) width: f32,
    pub(super) mandatory_break_after: bool,
}
