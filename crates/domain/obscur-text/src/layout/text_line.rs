//! One laid-out visual line of a paragraph (spec §5.10).

use obscur_geometry::{Point, Rect, Size};

use crate::shaping::GlyphRun;
use crate::span::TextRange;

/// A single laid-out visual line: the glyph runs it contains plus the geometry
/// placing it within the paragraph (spec §5.10).
///
/// A line owns one or more [`GlyphRun`]s (a single run in the common,
/// already-itemized case), the source byte [`range`](Self::range) it covers, and
/// the metrics needed to position it: [`top`](Self::top) is the y of the line
/// box's top edge within the paragraph, [`height`](Self::height) its box height,
/// [`baseline`](Self::baseline) the baseline offset measured down from `top`, and
/// [`x_offset`](Self::x_offset) the horizontal start offset produced by alignment.
/// [`width`](Self::width) is the sum of the runs' advances and is computed at
/// construction. Because it embeds `f32` metrics and non-`Eq` glyph runs it
/// derives `PartialEq` but not `Eq`.
#[derive(Debug, Clone, PartialEq)]
pub struct TextLine {
    /// The glyph runs composing the line, in visual order.
    runs: Vec<GlyphRun>,
    /// The byte range of the source text this line covers.
    range: TextRange,
    /// The y of the line box's top edge within the paragraph, in logical pixels.
    top: f32,
    /// The sum of the runs' advance widths, in logical pixels.
    width: f32,
    /// The line box height, in logical pixels.
    height: f32,
    /// The baseline offset measured down from `top`, in logical pixels.
    baseline: f32,
    /// The horizontal start offset from alignment, in logical pixels.
    x_offset: f32,
}

impl TextLine {
    /// Creates a line from its `runs`, source `range`, box `top`, box `height`,
    /// and `baseline` offset.
    ///
    /// The line [`width`](Self::width) is computed as the sum of the runs'
    /// advances, and [`x_offset`](Self::x_offset) starts at `0` — set it with
    /// [`with_x_offset`](Self::with_x_offset) once alignment is resolved.
    #[must_use]
    pub fn new(runs: Vec<GlyphRun>, range: TextRange, top: f32, height: f32, baseline: f32) -> Self {
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

    /// Returns this line with its horizontal start offset set to `x_offset`.
    #[must_use]
    pub fn with_x_offset(mut self, x_offset: f32) -> Self {
        self.x_offset = x_offset;
        self
    }

    /// Returns the glyph runs composing the line, in visual order.
    #[must_use]
    pub fn runs(&self) -> &[GlyphRun] {
        &self.runs
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

    /// Returns the sum of the runs' advance widths, in logical pixels.
    #[must_use]
    pub const fn width(&self) -> f32 {
        self.width
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

    /// Returns the total number of glyphs across all of the line's runs.
    #[must_use]
    pub fn glyph_count(&self) -> usize {
        self.runs.iter().map(GlyphRun::len).sum()
    }

    /// Returns the line's box as a [`Rect`] positioned at
    /// `(x_offset, top)` and sized `width` × `height` (spec §5.10).
    #[must_use]
    pub fn bounds(&self) -> Rect {
        Rect::new(
            Point::new(self.x_offset, self.top),
            Size::new(self.width, self.height),
        )
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
    use crate::shaping::{ShapedGlyph, TextDirection};

    fn run(clusters: &[(usize, f32)]) -> GlyphRun {
        let glyphs = clusters
            .iter()
            .map(|&(cluster, adv)| ShapedGlyph::new(1, cluster, adv))
            .collect();
        GlyphRun::new(glyphs, TextDirection::LeftToRight, 10.0, TextRange::new(0, 3))
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
