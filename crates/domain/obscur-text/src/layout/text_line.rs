#![doc = include_str!("text_line.doc/_module.md")]

use obscur_geometry::{Point, Rect, Size};

use crate::shaping::GlyphRun;
use crate::span::TextRange;

#[doc = include_str!("text_line.doc/TextLine.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct TextLine {
    #[doc = include_str!("text_line.doc/runs.md")]
    runs: Vec<GlyphRun>,
    #[doc = include_str!("text_line.doc/range.md")]
    range: TextRange,
    #[doc = include_str!("text_line.doc/top.md")]
    top: f32,
    #[doc = include_str!("text_line.doc/width.md")]
    width: f32,
    #[doc = include_str!("text_line.doc/height.md")]
    height: f32,
    #[doc = include_str!("text_line.doc/baseline.md")]
    baseline: f32,
    #[doc = include_str!("text_line.doc/x_offset.md")]
    x_offset: f32,
}

impl TextLine {
    #[doc = include_str!("text_line.doc/new.md")]
    #[must_use]
    pub fn new(
        runs: Vec<GlyphRun>,
        range: TextRange,
        top: f32,
        height: f32,
        baseline: f32,
    ) -> Self {
        let width = runs.iter().map(GlyphRun::advance).sum();
        Self {
            runs,
            range,
            top,
            width,
            height,
            baseline,
            x_offset: 0.0,
        }
    }

    #[doc = include_str!("text_line.doc/with_x_offset.md")]
    #[must_use]
    pub fn with_x_offset(mut self, x_offset: f32) -> Self {
        self.x_offset = x_offset;
        self
    }

    #[doc = include_str!("text_line.doc/runs_2.md")]
    #[must_use]
    pub fn runs(&self) -> &[GlyphRun] {
        &self.runs
    }

    #[doc = include_str!("text_line.doc/range_2.md")]
    #[must_use]
    pub const fn range(&self) -> TextRange {
        self.range
    }

    #[doc = include_str!("text_line.doc/top_2.md")]
    #[must_use]
    pub const fn top(&self) -> f32 {
        self.top
    }

    #[doc = include_str!("text_line.doc/width_2.md")]
    #[must_use]
    pub const fn width(&self) -> f32 {
        self.width
    }

    #[doc = include_str!("text_line.doc/height_2.md")]
    #[must_use]
    pub const fn height(&self) -> f32 {
        self.height
    }

    #[doc = include_str!("text_line.doc/baseline_2.md")]
    #[must_use]
    pub const fn baseline(&self) -> f32 {
        self.baseline
    }

    #[doc = include_str!("text_line.doc/x_offset_2.md")]
    #[must_use]
    pub const fn x_offset(&self) -> f32 {
        self.x_offset
    }

    #[doc = include_str!("text_line.doc/glyph_count.md")]
    #[must_use]
    pub fn glyph_count(&self) -> usize {
        self.runs.iter().map(GlyphRun::len).sum()
    }

    #[doc = include_str!("text_line.doc/bounds.md")]
    #[must_use]
    pub fn bounds(&self) -> Rect {
        Rect::new(
            Point::new(self.x_offset, self.top),
            Size::new(self.width, self.height),
        )
    }

    #[doc = include_str!("text_line.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.glyph_count() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shaping::{ShapedGlyph, TextDirection};

    fn run(clusters: &[(usize, f32)]) -> GlyphRun {
        let glyphs = clusters
            .iter()
            .map(|&(cluster, adv)| ShapedGlyph::new(1, cluster, adv))
            .collect();
        GlyphRun::new(
            glyphs,
            TextDirection::LeftToRight,
            10.0,
            TextRange::new(0, 3),
        )
    }

    #[test]
    fn width_is_sum_of_advances() {
        let line = TextLine::new(
            vec![run(&[(0, 6.0), (1, 6.0), (2, 6.0)])],
            TextRange::new(0, 3),
            0.0,
            12.0,
            9.0,
        );
        assert_eq!(line.width(), 18.0);
        assert_eq!(line.glyph_count(), 3);
        assert!(!line.is_empty());
        assert_eq!(line.baseline(), 9.0);
    }

    #[test]
    fn bounds_uses_x_offset_and_top() {
        let line = TextLine::new(
            vec![run(&[(0, 6.0), (1, 6.0)])],
            TextRange::new(0, 2),
            5.0,
            12.0,
            9.0,
        )
        .with_x_offset(4.0);
        let bounds = line.bounds();
        assert_eq!(bounds.origin, Point::new(4.0, 5.0));
        assert_eq!(bounds.size, Size::new(12.0, 12.0));
        assert_eq!(line.x_offset(), 4.0);
    }

    #[test]
    fn empty_line_has_no_glyphs() {
        let line = TextLine::new(Vec::new(), TextRange::empty(0), 0.0, 12.0, 9.0);
        assert!(line.is_empty());
        assert_eq!(line.width(), 0.0);
        assert_eq!(line.glyph_count(), 0);
    }
}
