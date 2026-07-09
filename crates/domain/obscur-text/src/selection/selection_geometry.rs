//! The visual highlight rectangles of a selection (spec §5.11).

use obscur_geometry::{Point, Rect, Size};

use crate::layout::{ParagraphLayout, ParagraphLine};
use crate::shaping::ShapedGlyph;

/// The visual highlight of a [`TextSelection`](super::TextSelection) as a set of
/// rectangles (spec §5.11 editing engine).
///
/// A selection that spans several wrapped lines is drawn as one partial
/// rectangle per covered line — the leading line from the start offset to its
/// trailing edge, whole intermediate lines, and the trailing line from its start
/// to the end offset. Each rectangle is derived from the layout's per-line caret
/// geometry (see the `caret_rect` idiom on the layout hit-tester); this type is
/// a pure container that stores those rectangles and answers spatial queries
/// over them. It performs no layout of its own.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SelectionGeometry {
    /// The highlight rectangles, one partial rect per covered line.
    rects: Vec<Rect>,
}

impl SelectionGeometry {
    /// Creates an empty geometry with no rectangles.
    #[must_use]
    pub const fn new() -> Self {
        Self { rects: Vec::new() }
    }

    /// Creates a geometry from an existing list of rectangles.
    #[must_use]
    pub const fn from_rects(rects: Vec<Rect>) -> Self {
        Self { rects }
    }

    /// Creates a geometry holding a single rectangle (a one-line selection).
    #[must_use]
    pub fn single(rect: Rect) -> Self {
        Self { rects: vec![rect] }
    }

    /// Appends a rectangle to the highlight.
    pub fn push(&mut self, rect: Rect) {
        self.rects.push(rect);
    }

    /// Returns the highlight rectangles in the order they were added.
    #[must_use]
    pub fn rects(&self) -> &[Rect] {
        &self.rects
    }

    /// Returns whether the geometry holds no rectangles.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }

    /// Returns the number of rectangles.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rects.len()
    }

    /// Returns the union bounding box of every rectangle, or `None` when the
    /// geometry is empty.
    ///
    /// The bounds run from the minimum origin (top-left) to the maximum right
    /// and bottom edges across all rectangles.
    #[must_use]
    pub fn bounds(&self) -> Option<Rect> {
        let (first, rest) = self.rects.split_first()?;
        let mut min_x = first.origin.x;
        let mut min_y = first.origin.y;
        let mut max_x = first.right();
        let mut max_y = first.bottom();
        for rect in rest {
            min_x = min_x.min(rect.origin.x);
            min_y = min_y.min(rect.origin.y);
            max_x = max_x.max(rect.right());
            max_y = max_y.max(rect.bottom());
        }
        Some(Rect::new(
            Point::new(min_x, min_y),
            Size::new(max_x - min_x, max_y - min_y),
        ))
    }

    /// Returns whether any rectangle contains `point`.
    #[must_use]
    pub fn contains_point(&self, point: Point) -> bool {
        self.rects.iter().any(|rect| rect.contains(point))
    }

    /// Builds the per-line highlight for the byte span `start..end` over a wrapped
    /// `layout` (spec §5.11 editing engine).
    ///
    /// A selection that spans several wrapped lines paints one rectangle per
    /// covered line: the **leading** line runs from the start offset's pen x to
    /// that line's trailing edge, whole **interior** lines span their full used
    /// width, and the **trailing** line runs from its content start to the end
    /// offset's pen x. The rectangles are in *paragraph-local* coordinates — x is
    /// the line's alignment [`x_offset`](ParagraphLine::x_offset) plus the
    /// within-line pen x, y is the line box's [`top`](ParagraphLine::top) — so the
    /// caller translates them by the text box origin (and any horizontal scroll)
    /// to place them on screen.
    ///
    /// The within-line pen x is the sum of THAT line's spaced/tabbed glyph
    /// advances whose source cluster falls before the offset — the same advances
    /// the line's glyphs paint from — so a span confined to ONE line yields exactly
    /// the single rectangle the caret-driven highlight produced before. A collapsed
    /// span (`start == end`) and any zero-width per-line extent (e.g. a downstream
    /// caret's sliver at a soft-wrap boundary, or an empty line) contribute no
    /// rectangle. Determinism: the covered lines and their advances are a pure
    /// function of the layout and the two offsets.
    #[must_use]
    pub fn from_paragraph(layout: &ParagraphLayout, start: usize, end: usize) -> Self {
        let (lo, hi) = (start.min(end), start.max(end));
        let lines = layout.lines();
        if lo == hi || lines.is_empty() {
            return Self::new();
        }
        let start_line = Self::line_of_byte(lines, lo);
        let end_line = Self::line_of_byte(lines, hi);
        let mut rects = Vec::with_capacity(end_line.saturating_sub(start_line) + 1);
        for (li, line) in lines.iter().enumerate().take(end_line + 1).skip(start_line) {
            // Leading line: from the start pen x; interior/trailing: from x = 0.
            let x_from = if li == start_line { Self::line_pen_x(line, lo) } else { 0.0 };
            // Trailing line: to the end pen x; leading/interior: to the line width.
            let x_to = if li == end_line { Self::line_pen_x(line, hi) } else { line.width() };
            let width = x_to - x_from;
            // Drop a zero/negative-width extent (a boundary sliver or empty line):
            // it paints nothing, keeping a one-line span byte-identical.
            if width > 0.0 {
                rects.push(Rect::new(
                    Point::new(line.x_offset() + x_from, line.top()),
                    Size::new(width, line.height()),
                ));
            }
        }
        Self { rects }
    }

    /// The line index a caret byte `index` sits on: the first line whose byte range
    /// **ends past** `index`, else the last line — the same downstream resolution
    /// the caret hit-test uses, so a selection edge lands on the line its glyphs
    /// paint on.
    fn line_of_byte(lines: &[ParagraphLine], index: usize) -> usize {
        lines
            .iter()
            .position(|line| index < line.range().end)
            .unwrap_or_else(|| lines.len().saturating_sub(1))
    }

    /// The within-line pen x of `index`: the sum of this `line`'s spaced/tabbed
    /// glyph advances whose source cluster falls before `index`, measured from the
    /// line's content start (the same advances the segments paint from).
    fn line_pen_x(line: &ParagraphLine, index: usize) -> f32 {
        line.segments()
            .iter()
            .flat_map(|segment| segment.run().glyphs())
            .filter(|glyph| glyph.cluster() < index)
            .map(ShapedGlyph::x_advance)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::FontId;
    use crate::layout::{TextAlignment, TextOverflow};
    use crate::shaping::{GlyphRun, ShapedGlyph, ShapedRun, TextDirection};
    use crate::span::TextRange;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(x, y), Size::new(w, h))
    }

    /// A one-segment line whose glyphs carry the given `(cluster, advance)` pairs,
    /// covering byte `range`, boxed at `top` with a 12px height, offset by `xoff`.
    fn line(
        clusters: &[(usize, f32)],
        range: TextRange,
        top: f32,
        xoff: f32,
    ) -> ParagraphLine {
        let glyphs = clusters
            .iter()
            .map(|&(cluster, adv)| ShapedGlyph::new(1, cluster, adv))
            .collect();
        let segment = ShapedRun::new(
            FontId::new(0),
            GlyphRun::new(glyphs, TextDirection::LeftToRight, 10.0, range),
        );
        ParagraphLine::new(vec![segment], range, top, 12.0, 9.0).with_x_offset(xoff)
    }

    /// A three-line layout — "aaaa" / "bbbb" / "cccc" — each glyph 10px wide, lines
    /// stacked by 12px, covering bytes 0..4 / 4..8 / 8..12.
    fn three_line_layout(xoffsets: [f32; 3]) -> ParagraphLayout {
        let lines = vec![
            line(&[(0, 10.0), (1, 10.0), (2, 10.0), (3, 10.0)], TextRange::new(0, 4), 0.0, xoffsets[0]),
            line(&[(4, 10.0), (5, 10.0), (6, 10.0), (7, 10.0)], TextRange::new(4, 8), 12.0, xoffsets[1]),
            line(&[(8, 10.0), (9, 10.0), (10, 10.0), (11, 10.0)], TextRange::new(8, 12), 24.0, xoffsets[2]),
        ];
        ParagraphLayout::new(lines, TextAlignment::Start, TextOverflow::Clip)
    }

    #[test]
    fn from_paragraph_collapsed_span_is_empty() {
        let layout = three_line_layout([0.0; 3]);
        assert!(SelectionGeometry::from_paragraph(&layout, 5, 5).is_empty());
    }

    #[test]
    fn from_paragraph_single_line_is_one_rect() {
        // A span [1,3) confined to line 0: one rect from pen x 10 (one glyph) to
        // pen x 30 (three glyphs) — width 20 — at the line's top and height.
        let layout = three_line_layout([0.0; 3]);
        let geo = SelectionGeometry::from_paragraph(&layout, 1, 3);
        assert_eq!(geo.len(), 1);
        assert_eq!(geo.rects()[0], rect(10.0, 0.0, 20.0, 12.0));
    }

    #[test]
    fn from_paragraph_spans_three_lines_with_per_line_extents() {
        // A span [2,10) covers three lines: start-line TAIL (pen x 20 → width 40),
        // interior FULL line, and end-line HEAD (0 → pen x 20).
        let layout = three_line_layout([0.0; 3]);
        let geo = SelectionGeometry::from_paragraph(&layout, 2, 10);
        assert_eq!(geo.len(), 3, "one rect per covered line");
        assert_eq!(geo.rects()[0], rect(20.0, 0.0, 20.0, 12.0), "start line: tail");
        assert_eq!(geo.rects()[1], rect(0.0, 12.0, 40.0, 12.0), "interior: full width");
        assert_eq!(geo.rects()[2], rect(0.0, 24.0, 20.0, 12.0), "end line: head");
    }

    #[test]
    fn from_paragraph_backward_span_normalizes() {
        // A backward span (focus < anchor) highlights the same rects as forward.
        let layout = three_line_layout([0.0; 3]);
        assert_eq!(
            SelectionGeometry::from_paragraph(&layout, 10, 2),
            SelectionGeometry::from_paragraph(&layout, 2, 10),
        );
    }

    #[test]
    fn from_paragraph_honors_line_alignment_offset() {
        // Each covered line's rect x includes that line's own alignment x_offset.
        let layout = three_line_layout([5.0, 7.0, 9.0]);
        let geo = SelectionGeometry::from_paragraph(&layout, 2, 10);
        assert_eq!(geo.rects()[0].origin.x, 5.0 + 20.0);
        assert_eq!(geo.rects()[1].origin.x, 7.0 + 0.0);
        assert_eq!(geo.rects()[2].origin.x, 9.0 + 0.0);
    }

    #[test]
    fn empty_defaults() {
        let geo = SelectionGeometry::new();
        assert!(geo.is_empty());
        assert_eq!(geo.len(), 0);
        assert_eq!(geo.bounds(), None);
        assert_eq!(geo, SelectionGeometry::default());
    }

    #[test]
    fn single_and_push() {
        let mut geo = SelectionGeometry::single(rect(0.0, 0.0, 10.0, 12.0));
        assert_eq!(geo.len(), 1);
        geo.push(rect(0.0, 12.0, 6.0, 12.0));
        assert_eq!(geo.len(), 2);
        assert_eq!(geo.rects().len(), 2);
    }

    #[test]
    fn bounds_unions_across_multiple_rects() {
        // A two-line selection: a wide first line and an offset second line.
        let geo = SelectionGeometry::from_rects(vec![
            rect(20.0, 0.0, 40.0, 12.0), // right edge 60, bottom 12
            rect(0.0, 12.0, 30.0, 12.0), // left edge 0, bottom 24
        ]);
        let bounds = geo.bounds().unwrap();
        assert_eq!(bounds.origin, Point::new(0.0, 0.0));
        assert_eq!(bounds.size, Size::new(60.0, 24.0));
    }

    #[test]
    fn contains_point_checks_any_rect() {
        let geo = SelectionGeometry::from_rects(vec![
            rect(0.0, 0.0, 10.0, 12.0),
            rect(0.0, 12.0, 6.0, 12.0),
        ]);
        // Inside the first rect.
        assert!(geo.contains_point(Point::new(5.0, 5.0)));
        // Inside the second rect only.
        assert!(geo.contains_point(Point::new(3.0, 18.0)));
        // In the gap to the right of the second line.
        assert!(!geo.contains_point(Point::new(8.0, 18.0)));
        // Outside everything.
        assert!(!geo.contains_point(Point::new(100.0, 100.0)));
    }
}
