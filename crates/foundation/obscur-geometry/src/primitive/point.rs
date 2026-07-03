//! A 2D point in logical pixels.

/// A point in logical-pixel space. Coordinate-space tagging is layered on top
/// by consumers (spec §4.5); this primitive is space-agnostic.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    /// Horizontal coordinate.
    pub x: f32,
    /// Vertical coordinate.
    pub y: f32,
}

impl Point {
    /// The origin (0, 0).
    pub const ORIGIN: Self = Self { x: 0.0, y: 0.0 };

    /// Constructs a point.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
