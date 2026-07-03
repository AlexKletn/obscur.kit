//! The arranged box of a node (skeleton).

use obscur_geometry::Rect;

/// The result of arranging a node: its rectangle in parent space (spec §4.2).
/// Fractional geometry is preserved until paint time (spec §4.6).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LayoutBox {
    /// Arranged bounds in parent-relative logical pixels.
    pub bounds: Rect,
}

impl LayoutBox {
    /// Creates a layout box from arranged bounds.
    #[must_use]
    pub const fn new(bounds: Rect) -> Self {
        Self { bounds }
    }
}
