//! A radial (concentric) gradient brush.

use obscur_geometry::Point;

use super::{ExtendMode, GradientStop};

/// A gradient whose color varies radially outward from `center` (spec §5.1).
///
/// Offset `0.0` maps to `center` and offset `1.0` maps to the circle of the
/// given `radius`; the color at any point depends on its distance from
/// `center`. Behavior beyond `radius` (positions past offset `1.0`) is governed
/// by the [`ExtendMode`].
///
/// This is a simple **concentric** radial gradient for Phase 1: the color rings
/// are centered on a single point. A distinct focal point (an offset origin for
/// the `0.0` ring, giving an elliptical/eccentric gradient) is a later
/// extension.
///
/// Stops are stored in ascending `offset` order (callers supply them sorted;
/// see [`GradientStop`]).
#[derive(Debug, Clone, PartialEq)]
pub struct RadialGradient {
    /// Center of the gradient, mapping to offset `0.0`.
    pub center: Point,
    /// Radius of the outermost ring, mapping to offset `1.0`.
    pub radius: f32,
    /// Color stops, ordered by ascending offset.
    pub stops: Vec<GradientStop>,
    /// How the gradient extends beyond its `[0, 1]` range.
    pub extend: ExtendMode,
}

impl RadialGradient {
    /// Creates a radial gradient centered at `center` with the given `radius`
    /// and `stops`, using the default [`ExtendMode::Pad`] extend mode.
    #[must_use]
    pub fn new(center: Point, radius: f32, stops: Vec<GradientStop>) -> Self {
        Self {
            center,
            radius,
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

    /// Returns the gradient's center point (offset `0.0`).
    #[must_use]
    pub const fn center(&self) -> Point {
        self.center
    }

    /// Returns the gradient's radius (offset `1.0`).
    #[must_use]
    pub const fn radius(&self) -> f32 {
        self.radius
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
        let g = RadialGradient::new(Point::ORIGIN, 10.0, stops());
        assert_eq!(g.center(), Point::ORIGIN);
        assert_eq!(g.radius(), 10.0);
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.extend(), ExtendMode::Pad);
    }

    #[test]
    fn with_extend_overrides() {
        let g = RadialGradient::new(Point::ORIGIN, 10.0, stops())
            .with_extend(ExtendMode::Reflect);
        assert_eq!(g.extend(), ExtendMode::Reflect);
    }
}
