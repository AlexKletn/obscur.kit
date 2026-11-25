//! One laid-out visual line of a multi-font paragraph (spec §5.10).

use crate::shaping::ShapedRun;
use crate::shaping::TextDirection;
use crate::span::TextRange;

#[doc = include_str!("../../docs/api/layout/paragraph_line.doc/ParagraphLine.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ParagraphLine {
    /// The contiguous same-font segments composing the line, in visual order.
    segments: Vec<ShapedRun>,
    /// The byte range of the source text this line covers.
    range: TextRange,
    /// The y of the line box's top edge within the paragraph, in logical pixels.
    top: f32,
    /// The line box height, in logical pixels.
    height: f32,
    /// The baseline offset measured down from `top`, in logical pixels.
    baseline: f32,
    /// The horizontal start offset from alignment, in logical pixels.
    x_offset: f32,
}

impl ParagraphLine {
    #[doc = include_str!("../../docs/api/layout/paragraph_line.doc/new.md")]
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

    /// Returns this line with its horizontal start offset set to `x_offset`.
    #[must_use]
    pub fn with_x_offset(mut self, x_offset: f32) -> Self {
        self.x_offset = x_offset;
        self
    }

    /// Returns the contiguous same-font segments composing the line, in visual
    /// order.
    #[must_use]
    pub fn segments(&self) -> &[ShapedRun] {
        &self.segments
    }

    /// Returns the byte range of the source text this line covers.
    #[must_use]
    pub const fn range(&self) -> TextRange {
        self.range
    }

    /// Returns the y of the line box's top edge within the paragraph.
    #[must_use]
    pub const fn top(&self) -> f32 {
        self.top
    }

    /// Returns the line box height, in logical pixels.
    #[must_use]
    pub const fn height(&self) -> f32 {
        self.height
    }

    /// Returns the baseline offset measured down from [`top`](Self::top).
    #[must_use]
    pub const fn baseline(&self) -> f32 {
        self.baseline
    }

    /// Returns the horizontal start offset from alignment, in logical pixels.
    #[must_use]
    pub const fn x_offset(&self) -> f32 {
        self.x_offset
    }

    #[doc = include_str!("../../docs/api/layout/paragraph_line.doc/direction.md")]
    #[must_use]
    pub fn direction(&self) -> TextDirection {
        self.segments
            .first()
            .map_or(TextDirection::LeftToRight, |segment| {
                segment.run().direction()
            })
    }

    /// Returns the line's total advance width: the sum of every segment's run
    /// advance, in logical pixels (spec §5.10).
    #[must_use]
    pub fn width(&self) -> f32 {
        self.segments
            .iter()
            .map(|segment| segment.run().advance())
            .sum()
    }

    /// Returns the total number of glyphs across all of the line's segments.
    #[must_use]
    pub fn glyph_count(&self) -> usize {
        self.segments
            .iter()
            .map(|segment| segment.run().len())
            .sum()
    }

    /// Returns whether the line contains no glyphs.
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
