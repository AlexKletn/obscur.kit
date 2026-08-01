#![doc = include_str!("glyph_run.doc/_module.md")]

use crate::shaping::{ShapedGlyph, TextDirection};
use crate::span::TextRange;

#[doc = include_str!("glyph_run.doc/GlyphRun.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct GlyphRun {
    #[doc = include_str!("glyph_run.doc/glyphs.md")]
    glyphs: Vec<ShapedGlyph>,
    #[doc = include_str!("glyph_run.doc/direction.md")]
    direction: TextDirection,
    #[doc = include_str!("glyph_run.doc/font_size.md")]
    font_size: f32,
    #[doc = include_str!("glyph_run.doc/source.md")]
    source: TextRange,
}

impl GlyphRun {
    #[doc = include_str!("glyph_run.doc/new.md")]
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

    #[doc = include_str!("glyph_run.doc/glyphs_2.md")]
    #[must_use]
    pub fn glyphs(&self) -> &[ShapedGlyph] {
        &self.glyphs
    }

    #[doc = include_str!("glyph_run.doc/direction_2.md")]
    #[must_use]
    pub const fn direction(&self) -> TextDirection {
        self.direction
    }

    #[doc = include_str!("glyph_run.doc/font_size_2.md")]
    #[must_use]
    pub const fn font_size(&self) -> f32 {
        self.font_size
    }

    #[doc = include_str!("glyph_run.doc/source_2.md")]
    #[must_use]
    pub const fn source(&self) -> TextRange {
        self.source
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

    #[doc = include_str!("glyph_run.doc/glyph_at.md")]
    #[must_use]
    pub fn glyph_at(&self, index: usize) -> Option<&ShapedGlyph> {
        self.glyphs.get(index)
    }

    #[doc = include_str!("glyph_run.doc/advance.md")]
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
