//! The computed value of CSS `background-position` (spec §5.2, CSS Backgrounds §3).

use crate::property::Length;

#[doc = include_str!("../../docs/api/property/background_position.doc/BackgroundPosition.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackgroundPosition {
    /// The horizontal position component (from the box's left edge).
    x: Length,
    /// The vertical position component (from the box's top edge).
    y: Length,
    from_end: [bool; 2],
}

impl BackgroundPosition {
    /// Builds a position from its per-axis components.
    #[must_use]
    pub const fn new(x: Length, y: Length) -> Self {
        Self::from_edge_offsets(x, y, [false; 2])
    }

    /// Positions an image using offsets from either end of each axis.
    #[must_use]
    pub const fn from_edge_offsets(x: Length, y: Length, from_end: [bool; 2]) -> Self {
        Self { x, y, from_end }
    }

    /// Whether the horizontal and vertical offsets start at the far edge.
    #[must_use]
    pub const fn end_edges(self) -> [bool; 2] {
        self.from_end
    }

    /// The horizontal position component.
    #[must_use]
    pub const fn x(self) -> Length {
        self.x
    }

    /// The vertical position component.
    #[must_use]
    pub const fn y(self) -> Length {
        self.y
    }
}

impl Default for BackgroundPosition {
    /// The CSS initial: `0% 0%` (top-left).
    fn default() -> Self {
        Self::new(Length::Percent(0.0), Length::Percent(0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_top_left() {
        let pos = BackgroundPosition::default();
        assert_eq!(pos.x(), Length::Percent(0.0));
        assert_eq!(pos.y(), Length::Percent(0.0));
    }

    #[test]
    fn new_round_trips_axes() {
        let pos = BackgroundPosition::new(Length::Percent(100.0), Length::px(10.0));
        assert_eq!(pos.x(), Length::Percent(100.0));
        assert_eq!(pos.y(), Length::px(10.0));
    }
}
