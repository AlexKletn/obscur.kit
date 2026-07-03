//! The arrange-phase input for a node.

use obscur_geometry::{Point, Rect, Size};

/// The **arrange** phase input for a node (spec §4.2): the final region
/// (a rectangle in parent-relative space) the parent assigns to the node.
///
/// Arrange positions a node (and recursively its children) into its assigned
/// region, producing a [`LayoutBox`](crate::model::LayoutBox). It runs after
/// the measure phase ([`MeasurePass`](super::MeasurePass)), which decides sizes
/// without positioning.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrangePass {
    /// The parent-relative region assigned to the node.
    region: Rect,
}

impl ArrangePass {
    /// Creates an arrange input carrying the assigned `region`.
    #[must_use]
    pub const fn new(region: Rect) -> Self {
        Self { region }
    }

    /// The parent-relative region assigned to the node.
    #[must_use]
    pub const fn region(self) -> Rect {
        self.region
    }

    /// The size of the assigned region.
    #[must_use]
    pub const fn size(self) -> Size {
        self.region.size
    }

    /// The origin of the assigned region, relative to the parent.
    #[must_use]
    pub const fn origin(self) -> Point {
        self.region.origin
    }
}

#[cfg(test)]
mod tests {
    use super::ArrangePass;
    use obscur_geometry::{Point, Rect, Size};

    #[test]
    fn exposes_region_size_and_origin() {
        let region = Rect::new(Point::new(4.0, 5.0), Size::new(20.0, 10.0));
        let pass = ArrangePass::new(region);
        assert_eq!(pass.region(), region);
        assert_eq!(pass.size(), Size::new(20.0, 10.0));
        assert_eq!(pass.origin(), Point::new(4.0, 5.0));
    }
}
