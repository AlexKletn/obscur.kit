use obscur_geometry::Size;

/// A child placed in the block-axis coordinate system of a vertical writing mode,
/// before it is mapped to a physical origin.
pub(super) struct VerticalPlacement {
    /// Distance from the container's block-start content edge to this child's own
    /// block-start edge, along the (horizontal) block axis.
    pub(super) block_start_offset: f32,
    /// Distance from the container's inline-start content edge (the top) to this
    /// child's top edge, along the (vertical) inline axis.
    pub(super) inline_offset: f32,
    /// The child's physical size (width along the block axis, height along inline).
    pub(super) size: Size,
}
