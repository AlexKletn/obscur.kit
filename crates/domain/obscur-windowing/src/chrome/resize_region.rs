//! A semantic resize area on a window edge or corner (spec §6.4).

use obscur_geometry::{Point, Rect};

use super::ResizeEdge;

#[doc = include_str!("../../docs/api/chrome/resize_region.doc/ResizeRegion.md")]
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
    fn region_contains_and_accessors() {
        let region = ResizeRegion::new(a_rect(), ResizeEdge::BottomRight);
        assert_eq!(region.edge(), ResizeEdge::BottomRight);
        assert_eq!(region.bounds(), a_rect());
        assert!(region.contains(Point::new(2.0, 2.0)));
        assert!(!region.contains(Point::new(9.0, 2.0)));
    }
}
