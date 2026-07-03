//! A linear (axial) gradient brush.

use obscur_geometry::Point;

use super::{ExtendMode, GradientStop};

/// A gradient whose color varies linearly along the axis from `start` to `end`
/// in user space (spec §5.1).
///
/// The gradient runs from `start` (offset `0.0`) to `end` (offset `1.0`); the
/// color at any point is found by projecting it onto that axis. Behavior for
/// projected positions outside `[0, 1]` is governed by the [`ExtendMode`].
///
/// Stops are stored in ascending `offset` order (callers supply them sorted;
/// see [`GradientStop`]).
#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    /// Axis start, mapping to gradient offset `0.0`.
    pub start: Point,
    /// Axis end, mapping to gradient offset `1.0`.
    pub end: Point,
    /// Color stops, ordered by ascending offset.
    pub stops: Vec<GradientStop>,
    /// How the gradient extends beyond its `[0, 1]` range.
    pub extend: ExtendMode,
}

impl LinearGradient {
    /// Creates a linear gradient from `start` to `end` with `stops` and the
    /// default [`ExtendMode::Pad`] extend mode.
    #[must_use]
    pub fn new(start: Point, end: Point, stops: Vec<GradientStop>) -> Self {
        Self {
            start,
            end,
            stops,
            extend: ExtendMode::Pad,
        }
    }

    /// Returns a copy of this gradient with its extend mode set to `mode`.
    #[must_use]
    pub fn with_extend(mut self, mode: ExtendMode) -> Self {
        self.extend = mode;
        self
    }

    /// Returns the axis start point (gradient offset `0.0`).
    #[must_use]
    pub const fn start(&self) -> Point {
        self.start
    }

    /// Returns the axis end point (gradient offset `1.0`).
    #[must_use]
    pub const fn end(&self) -> Point {
        self.end
    }

    /// Returns the gradient's color stops.
    #[must_use]
    pub fn stops(&self) -> &[GradientStop] {
        &self.stops
    }

    /// Returns the gradient's extend mode.
    #[must_use]
    pub const fn extend(&self) -> ExtendMode {
        self.extend
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    fn stops() -> Vec<GradientStop> {
        vec![
            GradientStop::new(0.0, Color::BLACK),
            GradientStop::new(1.0, Color::WHITE),
        ]
    }

    #[test]
    fn new_defaults_to_pad() {
        let g = LinearGradient::new(Point::ORIGIN, Point::new(1.0, 0.0), stops());
        assert_eq!(g.start(), Point::ORIGIN);
        assert_eq!(g.end(), Point::new(1.0, 0.0));
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.extend(), ExtendMode::Pad);
    }

    #[test]
    fn with_extend_overrides() {
        let g = LinearGradient::new(Point::ORIGIN, Point::new(1.0, 0.0), stops())
            .with_extend(ExtendMode::Repeat);
        assert_eq!(g.extend(), ExtendMode::Repeat);
    }
}
