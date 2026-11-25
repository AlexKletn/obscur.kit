use obscur_geometry::Rect;

/// A single indexed node: its visual `bounds` and `z_order` (spec §4.7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Entry {
    /// The node's already-resolved visual bounds (transform/clip applied
    /// upstream).
    pub bounds: Rect,
    /// The node's z-order; higher values are drawn on top.
    pub z_order: i32,
}

impl Entry {
    /// Constructs an entry from resolved `bounds` and a `z_order`.
    #[must_use]
    pub const fn new(bounds: Rect, z_order: i32) -> Self {
        Self { bounds, z_order }
    }
}
