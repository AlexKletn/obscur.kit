//! A semantic resize area on a window edge or corner (spec §6.4).

use obscur_geometry::{Point, Rect};

/// Which edge or corner of the window a resize region drives (spec §6.4).
///
/// `ResizeEdge` is a tightly-coupled part of [`ResizeRegion`]: it names the grab
/// direction of a resize area and has no meaning apart from it, so it is
/// colocated here rather than given its own module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResizeEdge {
    /// The top edge.
    Top,
    /// The bottom edge.
    Bottom,
    /// The left edge.
    Left,
    /// The right edge.
    Right,
    /// The top-left corner.
    TopLeft,
    /// The top-right corner.
    TopRight,
    /// The bottom-left corner.
    BottomLeft,
    /// The bottom-right corner.
    BottomRight,
}

impl ResizeEdge {
    /// Returns whether this edge is a corner (resizes in both axes).
    #[must_use]
    pub const fn is_corner(self) -> bool {
        matches!(
            self,
            Self::TopLeft | Self::TopRight | Self::BottomLeft | Self::BottomRight
        )
    }

    /// Returns whether resizing this edge changes the window's width.
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

    /// Returns whether resizing this edge changes the window's height.
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

/// A semantic resize area on one edge or corner of a window (spec §6.4).
///
/// A compositor treats a drag that begins within [`Self::bounds`] as a resize
/// in the direction named by [`Self::edge`]. Coordinates are in window-local
/// logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResizeRegion {
    /// The window-local area treated as a resize grab handle.
    pub bounds: Rect,
    /// The edge or corner this region resizes.
    pub edge: ResizeEdge,
}

impl ResizeRegion {
    /// Creates a resize region over `bounds` driving `edge`.
    #[must_use]
    pub const fn new(bounds: Rect, edge: ResizeEdge) -> Self {
        Self { bounds, edge }
    }

    /// Returns the window-local grab area.
    #[must_use]
    pub const fn bounds(&self) -> Rect {
        self.bounds
    }

    /// Returns the edge or corner this region resizes.
    #[must_use]
    pub const fn edge(&self) -> ResizeEdge {
        self.edge
    }

    /// Returns whether the window-local point lies within the resize region.
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
