#![doc = include_str!("path_segment.doc/_module.md")]

use obscur_geometry::Point;

#[doc = include_str!("path_segment.doc/PathSegment.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathSegment {
    #[doc = include_str!("path_segment.doc/MoveTo.md")]
    MoveTo(Point),
    #[doc = include_str!("path_segment.doc/LineTo.md")]
    LineTo(Point),
    #[doc = include_str!("path_segment.doc/QuadTo.md")]
    QuadTo {
        #[doc = include_str!("path_segment.doc/control.md")]
        control: Point,
        #[doc = include_str!("path_segment.doc/end.md")]
        end: Point,
    },
    #[doc = include_str!("path_segment.doc/CubicTo.md")]
    CubicTo {
        #[doc = include_str!("path_segment.doc/control1.md")]
        control1: Point,
        #[doc = include_str!("path_segment.doc/control2.md")]
        control2: Point,
        #[doc = include_str!("path_segment.doc/end_2.md")]
        end: Point,
    },
    #[doc = include_str!("path_segment.doc/Close.md")]
    Close,
}

impl PathSegment {
    #[doc = include_str!("path_segment.doc/end_point.md")]
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
