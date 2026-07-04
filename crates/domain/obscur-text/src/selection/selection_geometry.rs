//! The visual highlight rectangles of a selection (spec §5.11).

use obscur_geometry::{Point, Rect, Size};

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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(x, y), Size::new(w, h))
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
