//! An axis-aligned rectangle in logical pixels.

use super::{Point, Size};

/// An axis-aligned rectangle described by its top-left origin and size.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    /// Top-left corner.
    pub origin: Point,
    /// Extent.
    pub size: Size,
}

impl Rect {
    /// Constructs a rectangle from an origin and size.
    #[must_use]
    pub const fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    /// The x coordinate of the right edge.
    #[must_use]
    pub fn right(self) -> f32 {
        self.origin.x + self.size.width
    }

    /// The y coordinate of the bottom edge.
    #[must_use]
    pub fn bottom(self) -> f32 {
        self.origin.y + self.size.height
    }

    #[doc = include_str!("../../docs/api/primitive/rect.doc/contains.md")]
    #[must_use]
    pub fn contains(self, p: Point) -> bool {
        p.x >= self.origin.x && p.x < self.right() && p.y >= self.origin.y && p.y < self.bottom()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(x, y), Size::new(w, h))
    }

    #[test]
    fn the_origin_corner_is_inside_and_the_other_three_are_not() {
        let r = rect(10.0, 20.0, 30.0, 40.0);
        assert!(
            r.contains(Point::new(10.0, 20.0)),
            "the origin pixel belongs to the rect; excluding it blanks every \
             widget's top-left corner"
        );
        assert!(!r.contains(Point::new(40.0, 20.0)));
        assert!(!r.contains(Point::new(10.0, 60.0)));
        assert!(!r.contains(Point::new(40.0, 60.0)));
    }

    #[test]
    fn the_leading_edges_are_inside_and_the_trailing_edges_are_outside() {
        // `[origin, right) x [origin, bottom)`: left and top are closed, right
        // and bottom are open.
        let r = rect(10.0, 20.0, 30.0, 40.0);
        assert!(r.contains(Point::new(10.0, 40.0)), "left edge is inside");
        assert!(r.contains(Point::new(25.0, 20.0)), "top edge is inside");
        assert!(!r.contains(Point::new(40.0, 40.0)), "right edge is outside");
        assert!(
            !r.contains(Point::new(25.0, 60.0)),
            "bottom edge is outside"
        );

        assert!(
            !r.contains(Point::new(9.75, 40.0)),
            "just left of the left edge"
        );
        assert!(
            !r.contains(Point::new(25.0, 19.75)),
            "just above the top edge"
        );
        assert!(
            r.contains(Point::new(39.75, 40.0)),
            "just inside the right edge"
        );
        assert!(
            r.contains(Point::new(25.0, 59.75)),
            "just inside the bottom edge"
        );
    }

    #[test]
    fn two_abutting_rects_claim_their_shared_edge_exactly_once() {
        // Half-openness is what stops the seam between adjacent rows or columns
        // from hit-testing as both of them.
        let left = rect(0.0, 0.0, 10.0, 10.0);
        let right = rect(10.0, 0.0, 10.0, 10.0);
        let on_vertical_seam = Point::new(10.0, 5.0);
        assert!(!left.contains(on_vertical_seam));
        assert!(right.contains(on_vertical_seam));

        let top = rect(0.0, 0.0, 10.0, 10.0);
        let bottom = rect(0.0, 10.0, 10.0, 10.0);
        let on_horizontal_seam = Point::new(5.0, 10.0);
        assert!(!top.contains(on_horizontal_seam));
        assert!(bottom.contains(on_horizontal_seam));
    }

    #[test]
    fn a_rect_with_no_area_contains_nothing() {
        let empty = rect(5.0, 5.0, 0.0, 0.0);
        assert!(!empty.contains(Point::new(5.0, 5.0)));
        assert!(!empty.contains(Point::ORIGIN));

        // A degenerate rect in one axis only is just as empty.
        assert!(!rect(5.0, 5.0, 0.0, 10.0).contains(Point::new(5.0, 7.0)));
        assert!(!rect(5.0, 5.0, 10.0, 0.0).contains(Point::new(7.0, 5.0)));
    }

    #[test]
    fn a_point_outside_on_any_side_is_not_contained() {
        let r = rect(10.0, 20.0, 30.0, 40.0);
        assert!(!r.contains(Point::new(0.0, 40.0)));
        assert!(!r.contains(Point::new(100.0, 40.0)));
        assert!(!r.contains(Point::new(25.0, 0.0)));
        assert!(!r.contains(Point::new(25.0, 100.0)));
        assert!(
            r.contains(Point::new(25.0, 40.0)),
            "the interior still hits"
        );
    }
}
