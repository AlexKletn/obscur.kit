#[doc = include_str!("../../docs/api/algorithm/docking_layout.doc/DockSide.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockSide {
    /// Dock against the top edge, spanning the remaining width.
    Top,
    /// Dock against the bottom edge, spanning the remaining width.
    Bottom,
    /// Dock against the left edge, spanning the remaining height.
    Left,
    /// Dock against the right edge, spanning the remaining height.
    Right,
    /// Fill the entire remaining space.
    Fill,
}
