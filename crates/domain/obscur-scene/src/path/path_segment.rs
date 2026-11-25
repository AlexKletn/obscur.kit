//! One command in a path outline.

use obscur_geometry::Point;

#[doc = include_str!("../../docs/api/path/path_segment.doc/PathSegment.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathSegment {
    /// Start a new subpath at the given point, moving the pen without drawing.
    MoveTo(Point),
    /// Draw a straight line from the current point to the given point.
    LineTo(Point),
    /// Draw a quadratic Bézier curve to `end` using a single `control` point.
    QuadTo {
        /// The off-curve control point.
        control: Point,
        /// The on-curve end point.
        end: Point,
    },
    /// Draw a cubic Bézier curve to `end` using `control1` and `control2`.
    CubicTo {
        /// The first off-curve control point (leaves the start point).
        control1: Point,
        /// The second off-curve control point (arrives at the end point).
        control2: Point,
        /// The on-curve end point.
        end: Point,
    },
    /// Close the current subpath with a straight line back to its start.
    Close,
}

impl PathSegment {
    #[doc = include_str!("../../docs/api/path/path_segment.doc/end_point.md")]
    #[must_use]
    pub fn end_point(&self) -> Option<Point> {
        match self {
            Self::MoveTo(p) | Self::LineTo(p) => Some(*p),
            Self::QuadTo { end, .. } | Self::CubicTo { end, .. } => Some(*end),
            Self::Close => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_point_of_each_variant() {
        let p = Point::new(3.0, 4.0);
        assert_eq!(PathSegment::MoveTo(p).end_point(), Some(p));
        assert_eq!(PathSegment::LineTo(p).end_point(), Some(p));
        assert_eq!(
            PathSegment::QuadTo {
                control: Point::ORIGIN,
                end: p
            }
            .end_point(),
            Some(p)
        );
        assert_eq!(
            PathSegment::CubicTo {
                control1: Point::ORIGIN,
                control2: Point::ORIGIN,
                end: p
            }
            .end_point(),
            Some(p)
        );
        assert_eq!(PathSegment::Close.end_point(), None);
    }
}
