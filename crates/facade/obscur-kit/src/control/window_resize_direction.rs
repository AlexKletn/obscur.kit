//! Directions for interactive window resizing.

/// The edge or corner that the user is dragging, in physical window coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum WindowResizeDirection {
    /// Top edge.
    North,
    /// Top-right corner.
    NorthEast,
    /// Right edge.
    East,
    /// Bottom-right corner.
    SouthEast,
    /// Bottom edge.
    South,
    /// Bottom-left corner.
    SouthWest,
    /// Left edge.
    West,
    /// Top-left corner.
    NorthWest,
}
