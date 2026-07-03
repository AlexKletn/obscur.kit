//! An axis-aligned rectangle in logical pixels.

use super::{Point, Size};

/// An axis-aligned rectangle described by its top-left origin and size.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    /// Top-left corner.
    pub origin: Point,
    /// Extent.
    pub size: Size,
}

impl Rect {
    /// Constructs a rectangle from an origin and size.
    #[must_use]
    pub const fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    /// The x coordinate of the right edge.
    #[must_use]
    pub fn right(self) -> f32 {
        self.origin.x + self.size.width
    }

    /// The y coordinate of the bottom edge.
    #[must_use]
    pub fn bottom(self) -> f32 {
        self.origin.y + self.size.height
    }

    /// Returns whether the (fractional) point lies within the rectangle.
    ///
    /// Uses exact fractional geometry — hit testing must not pre-round
    /// (spec §4.6).
    #[must_use]
    pub fn contains(self, p: Point) -> bool {
        p.x >= self.origin.x && p.x < self.right() && p.y >= self.origin.y && p.y < self.bottom()
    }
}
