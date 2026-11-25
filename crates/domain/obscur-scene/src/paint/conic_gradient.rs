//! A conic (angular / sweep) gradient brush.

use obscur_geometry::Point;

use super::{ExtendMode, GradientStop};

#[doc = include_str!("../../docs/api/paint/conic_gradient.doc/ConicGradient.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ConicGradient {
    /// Center of the sweep.
    pub center: Point,
    /// Angle (radians) at which the sweep begins, mapping to offset `0.0`.
    pub start_angle: f32,
    /// Color stops, ordered by ascending offset.
    pub stops: Vec<GradientStop>,
    /// How the gradient extends beyond its `[0, 1]` range.
    pub extend: ExtendMode,
}

impl ConicGradient {
    /// Creates a conic gradient centered at `center`, sweeping clockwise from
    /// `start_angle` (radians), with `stops` and the default
    /// [`ExtendMode::Pad`] extend mode.
    #[must_use]
    pub fn new(center: Point, start_angle: f32, stops: Vec<GradientStop>) -> Self {
        Self {
            center,
            start_angle,
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

    /// Returns the sweep's center point.
    #[must_use]
    pub const fn center(&self) -> Point {
        self.center
    }

    /// Returns the angle (radians) at which the sweep begins (offset `0.0`).
    #[must_use]
    pub const fn start_angle(&self) -> f32 {
        self.start_angle
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
            GradientStop::new(0.0, Color::WHITE),
            GradientStop::new(1.0, Color::BLACK),
        ]
    }

    #[test]
    fn new_defaults_to_pad() {
        let g = ConicGradient::new(Point::ORIGIN, 0.0, stops());
        assert_eq!(g.center(), Point::ORIGIN);
        assert_eq!(g.start_angle(), 0.0);
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.extend(), ExtendMode::Pad);
    }

    #[test]
    fn with_extend_overrides() {
        let g = ConicGradient::new(Point::ORIGIN, 1.5, stops()).with_extend(ExtendMode::Repeat);
        assert_eq!(g.extend(), ExtendMode::Repeat);
    }
}
