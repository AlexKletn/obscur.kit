#![doc = include_str!("paragraph_line.doc/_module.md")]

use crate::shaping::ShapedRun;
use crate::shaping::TextDirection;
use crate::span::TextRange;

#[doc = include_str!("paragraph_line.doc/ParagraphLine.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ParagraphLine {
    #[doc = include_str!("paragraph_line.doc/segments.md")]
    segments: Vec<ShapedRun>,
    #[doc = include_str!("paragraph_line.doc/range.md")]
    range: TextRange,
    #[doc = include_str!("paragraph_line.doc/top.md")]
    top: f32,
    #[doc = include_str!("paragraph_line.doc/height.md")]
    height: f32,
    #[doc = include_str!("paragraph_line.doc/baseline.md")]
    baseline: f32,
    #[doc = include_str!("paragraph_line.doc/x_offset.md")]
    x_offset: f32,
}

impl ParagraphLine {
    #[doc = include_str!("paragraph_line.doc/new.md")]
    #[must_use]
    pub fn new(
        segments: Vec<ShapedRun>,
        range: TextRange,
        top: f32,
        height: f32,
        baseline: f32,
    ) -> Self {
        Self {
            segments,
            range,
            top,
            height,
            baseline,
            x_offset: 0.0,
        }
    }

    #[doc = include_str!("paragraph_line.doc/with_x_offset.md")]
    #[must_use]
    pub fn with_x_offset(mut self, x_offset: f32) -> Self {
        self.x_offset = x_offset;
        self
    }

    #[doc = include_str!("paragraph_line.doc/segments_2.md")]
    #[must_use]
    pub fn segments(&self) -> &[ShapedRun] {
        &self.segments
    }

    #[doc = include_str!("paragraph_line.doc/range_2.md")]
    #[must_use]
    pub const fn range(&self) -> TextRange {
        self.range
    }

    #[doc = include_str!("paragraph_line.doc/top_2.md")]
    #[must_use]
    pub const fn top(&self) -> f32 {
        self.top
    }

    #[doc = include_str!("paragraph_line.doc/height_2.md")]
    #[must_use]
    pub const fn height(&self) -> f32 {
        self.height
    }

    #[doc = include_str!("paragraph_line.doc/baseline_2.md")]
    #[must_use]
    pub const fn baseline(&self) -> f32 {
        self.baseline
    }

    #[doc = include_str!("paragraph_line.doc/x_offset_2.md")]
    #[must_use]
    pub const fn x_offset(&self) -> f32 {
        self.x_offset
    }

    #[doc = include_str!("paragraph_line.doc/direction.md")]
    #[must_use]
    pub fn direction(&self) -> TextDirection {
        self.segments
            .first()
            .map_or(TextDirection::LeftToRight, |segment| {
                segment.run().direction()
            })
    }

    #[doc = include_str!("paragraph_line.doc/width.md")]
    #[must_use]
    pub fn width(&self) -> f32 {
        self.segments
            .iter()
            .map(|segment| segment.run().advance())
            .sum()
    }

    #[doc = include_str!("paragraph_line.doc/glyph_count.md")]
    #[must_use]
    pub fn glyph_count(&self) -> usize {
        self.segments
            .iter()
            .map(|segment| segment.run().len())
            .sum()
    }

    #[doc = include_str!("paragraph_line.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.glyph_count() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::FontId;
    use crate::shaping::{GlyphRun, ShapedGlyph, TextDirection};

    fn segment(font: u32, clusters: &[(usize, f32)], range: TextRange) -> ShapedRun {
        let glyphs = clusters
            .iter()
            .map(|&(cluster, adv)| ShapedGlyph::new(1, cluster, adv))
            .collect();
        ShapedRun::new(
            FontId::new(font),
            GlyphRun::new(glyphs, TextDirection::LeftToRight, 10.0, range),
        )
    }

    #[test]
    fn width_and_glyph_count_sum_over_segments() {
        let line = ParagraphLine::new(
            vec![
                segment(0, &[(0, 6.0), (1, 6.0)], TextRange::new(0, 2)),
                segment(1, &[(2, 4.0)], TextRange::new(2, 3)),
            ],
            TextRange::new(0, 3),
            0.0,
            12.0,
            9.0,
        );
        assert_eq!(line.width(), 16.0);
        assert_eq!(line.glyph_count(), 3);
        assert!(!line.is_empty());
        assert_eq!(line.baseline(), 9.0);
        assert_eq!(line.segments().len(), 2);
    }

    #[test]
    fn with_x_offset_sets_the_offset() {
        let line = ParagraphLine::new(
            vec![segment(0, &[(0, 6.0)], TextRange::new(0, 1))],
            TextRange::new(0, 1),
            5.0,
            12.0,
            9.0,
        )
        .with_x_offset(4.0);
        assert_eq!(line.x_offset(), 4.0);
        assert_eq!(line.top(), 5.0);
        assert_eq!(line.height(), 12.0);
    }

    #[test]
    fn empty_line_has_no_glyphs() {
        let line = ParagraphLine::new(Vec::new(), TextRange::empty(0), 0.0, 12.0, 9.0);
        assert!(line.is_empty());
        assert_eq!(line.width(), 0.0);
        assert_eq!(line.glyph_count(), 0);
    }
}
