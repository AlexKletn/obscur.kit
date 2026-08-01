#![doc = include_str!("resize_region.doc/_module.md")]

use obscur_geometry::{Point, Rect};

#[doc = include_str!("resize_region.doc/ResizeEdge.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResizeEdge {
    #[doc = include_str!("resize_region.doc/Top.md")]
    Top,
    #[doc = include_str!("resize_region.doc/Bottom.md")]
    Bottom,
    #[doc = include_str!("resize_region.doc/Left.md")]
    Left,
    #[doc = include_str!("resize_region.doc/Right.md")]
    Right,
    #[doc = include_str!("resize_region.doc/TopLeft.md")]
    TopLeft,
    #[doc = include_str!("resize_region.doc/TopRight.md")]
    TopRight,
    #[doc = include_str!("resize_region.doc/BottomLeft.md")]
    BottomLeft,
    #[doc = include_str!("resize_region.doc/BottomRight.md")]
    BottomRight,
}

impl ResizeEdge {
    #[doc = include_str!("resize_region.doc/is_corner.md")]
    #[must_use]
    pub const fn is_corner(self) -> bool {
        matches!(
            self,
            Self::TopLeft | Self::TopRight | Self::BottomLeft | Self::BottomRight
        )
    }

    #[doc = include_str!("resize_region.doc/is_horizontal.md")]
    #[must_use]
    pub const fn is_horizontal(self) -> bool {
        matches!(
            self,
            Self::Left
                | Self::Right
                | Self::TopLeft
                | Self::TopRight
                | Self::BottomLeft
                | Self::BottomRight
        )
    }

    #[doc = include_str!("resize_region.doc/is_vertical.md")]
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        matches!(
            self,
            Self::Top
                | Self::Bottom
                | Self::TopLeft
                | Self::TopRight
                | Self::BottomLeft
                | Self::BottomRight
        )
    }
}

#[doc = include_str!("resize_region.doc/ResizeRegion.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResizeRegion {
    #[doc = include_str!("resize_region.doc/bounds.md")]
    pub bounds: Rect,
    #[doc = include_str!("resize_region.doc/edge.md")]
    pub edge: ResizeEdge,
}

impl ResizeRegion {
    #[doc = include_str!("resize_region.doc/new.md")]
    #[must_use]
    pub const fn new(bounds: Rect, edge: ResizeEdge) -> Self {
        Self { bounds, edge }
    }

    #[doc = include_str!("resize_region.doc/bounds_2.md")]
    #[must_use]
    pub const fn bounds(&self) -> Rect {
        self.bounds
    }

    #[doc = include_str!("resize_region.doc/edge_2.md")]
    #[must_use]
    pub const fn edge(&self) -> ResizeEdge {
        self.edge
    }

    #[doc = include_str!("resize_region.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, point: Point) -> bool {
        self.bounds.contains(point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Size;

    fn a_rect() -> Rect {
        Rect::new(Point::new(0.0, 0.0), Size::new(8.0, 8.0))
    }

    #[test]
    fn is_corner_only_for_corners() {
        assert!(ResizeEdge::TopLeft.is_corner());
        assert!(ResizeEdge::BottomRight.is_corner());
        assert!(!ResizeEdge::Top.is_corner());
        assert!(!ResizeEdge::Left.is_corner());
    }

    #[test]
    fn axis_predicates() {
        assert!(ResizeEdge::Left.is_horizontal());
        assert!(!ResizeEdge::Left.is_vertical());
        assert!(ResizeEdge::Top.is_vertical());
        assert!(!ResizeEdge::Top.is_horizontal());
        assert!(ResizeEdge::TopLeft.is_horizontal());
        assert!(ResizeEdge::TopLeft.is_vertical());
    }

    #[test]
    fn region_contains_and_accessors() {
        let region = ResizeRegion::new(a_rect(), ResizeEdge::BottomRight);
        assert_eq!(region.edge(), ResizeEdge::BottomRight);
        assert_eq!(region.bounds(), a_rect());
        assert!(region.contains(Point::new(2.0, 2.0)));
        assert!(!region.contains(Point::new(9.0, 2.0)));
    }
}
