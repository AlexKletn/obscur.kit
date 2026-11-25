//! The shaped glyph output of one text run (spec §5.10).

use crate::shaping::{ShapedGlyph, TextDirection};
use crate::span::TextRange;

#[doc = include_str!("../../docs/api/shaping/glyph_run.doc/GlyphRun.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct GlyphRun {
    /// The shaped glyphs, in pen order.
    glyphs: Vec<ShapedGlyph>,
    /// The base direction the run was shaped for.
    direction: TextDirection,
    /// The font size the run was shaped at, in logical pixels.
    font_size: f32,
    /// The byte range of the source text this run covers.
    source: TextRange,
}

impl GlyphRun {
    /// Creates a run from its shaped `glyphs`, base `direction`, `font_size`, and
    /// the `source` byte range it covers.
    #[must_use]
    pub const fn new(
        glyphs: Vec<ShapedGlyph>,
        direction: TextDirection,
        font_size: f32,
        source: TextRange,
    ) -> Self {
        Self {
            glyphs,
            direction,
            font_size,
            source,
        }
    }

    /// Returns the shaped glyphs, in pen order.
    #[must_use]
    pub fn glyphs(&self) -> &[ShapedGlyph] {
        &self.glyphs
    }

    /// Returns the base direction the run was shaped for.
    #[must_use]
    pub const fn direction(&self) -> TextDirection {
        self.direction
    }

    /// Returns the font size the run was shaped at, in logical pixels.
    #[must_use]
    pub const fn font_size(&self) -> f32 {
        self.font_size
    }

    /// Returns the byte range of the source text this run covers.
    #[must_use]
    pub const fn source(&self) -> TextRange {
        self.source
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

    /// Returns the glyph at index `index`, or `None` if out of bounds.
    #[must_use]
    pub fn glyph_at(&self, index: usize) -> Option<&ShapedGlyph> {
        self.glyphs.get(index)
    }

    /// Returns the run's total advance width: the sum of every glyph's
    /// [`x_advance`](ShapedGlyph::x_advance), in logical pixels (spec §5.10).
    #[must_use]
    pub fn advance(&self) -> f32 {
        self.glyphs.iter().map(ShapedGlyph::x_advance).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_run() -> GlyphRun {
        let glyphs = vec![
            ShapedGlyph::new(1, 0, 6.0),
            ShapedGlyph::new(2, 1, 6.0),
            ShapedGlyph::new(3, 2, 6.0),
        ];
        GlyphRun::new(
            glyphs,
            TextDirection::LeftToRight,
            10.0,
            TextRange::new(0, 3),
        )
    }

    #[test]
    fn advance_sums_glyph_advances() {
        assert_eq!(sample_run().advance(), 18.0);
    }

    #[test]
    fn glyph_at_indexes_and_bounds_check() {
        let run = sample_run();
        assert_eq!(run.glyph_at(0), Some(&ShapedGlyph::new(1, 0, 6.0)));
        assert_eq!(run.glyph_at(2), Some(&ShapedGlyph::new(3, 2, 6.0)));
        assert_eq!(run.glyph_at(3), None);
    }

    #[test]
    fn len_source_and_accessors() {
        let run = sample_run();
        assert_eq!(run.len(), 3);
        assert!(!run.is_empty());
        assert_eq!(run.direction(), TextDirection::LeftToRight);
        assert_eq!(run.font_size(), 10.0);
        assert_eq!(run.source(), TextRange::new(0, 3));
    }

    #[test]
    fn empty_run_has_zero_advance() {
        let run = GlyphRun::new(
            Vec::new(),
            TextDirection::LeftToRight,
            10.0,
            TextRange::empty(0),
        );
        assert!(run.is_empty());
        assert_eq!(run.len(), 0);
        assert_eq!(run.advance(), 0.0);
        assert_eq!(run.glyph_at(0), None);
    }
}
