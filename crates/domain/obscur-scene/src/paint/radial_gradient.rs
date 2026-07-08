//! A radial (concentric) gradient brush.

use obscur_geometry::Point;

use super::{ExtendMode, GradientStop};

/// A gradient whose color varies radially outward from `center` (spec §5.1).
///
/// Offset `0.0` maps to `center` and offset `1.0` maps to the ending shape of
/// horizontal radius `radius` and vertical radius `radius_y`; the color at any
/// point depends on its normalized distance from `center`. Behavior beyond the
/// ending shape (positions past offset `1.0`) is governed by the [`ExtendMode`].
///
/// The ending shape is a **circle** when `radius_y == radius` and an axis-aligned
/// **ellipse** otherwise (`radius` is the x-axis radius, `radius_y` the y-axis
/// radius) — see [CSS Images §3.2]. Color-stop offsets are measured along the
/// x-axis radius; a backend renders the ellipse by stretching the concentric
/// circle of radius `radius` vertically by `radius_y / radius`.
///
/// This is a simple **concentric** radial gradient: the color rings are centered
/// on a single point. A distinct focal point (an offset origin for the `0.0`
/// ring) is a later extension.
///
/// Stops are stored in ascending `offset` order (callers supply them sorted;
/// see [`GradientStop`]).
///
/// [CSS Images §3.2]: https://drafts.csswg.org/css-images-3/#radial-gradients
#[derive(Debug, Clone, PartialEq)]
pub struct RadialGradient {
    /// Center of the gradient, mapping to offset `0.0`.
    pub center: Point,
    /// Horizontal (x-axis) radius of the ending shape, mapping to offset `1.0`.
    pub radius: f32,
    /// Vertical (y-axis) radius of the ending shape. Equal to `radius` for a
    /// circle; different for an ellipse.
    pub radius_y: f32,
    /// Color stops, ordered by ascending offset.
    pub stops: Vec<GradientStop>,
    /// How the gradient extends beyond its `[0, 1]` range.
    pub extend: ExtendMode,
}

impl RadialGradient {
    /// Creates a **circular** radial gradient centered at `center` with the given
    /// `radius` and `stops`, using the default [`ExtendMode::Pad`] extend mode.
    #[must_use]
    pub fn new(center: Point, radius: f32, stops: Vec<GradientStop>) -> Self {
        Self {
            center,
            radius,
            radius_y: radius,
            stops,
            extend: ExtendMode::Pad,
        }
    }

    /// Creates an axis-aligned **elliptical** radial gradient centered at `center`
    /// with x-axis radius `radius_x`, y-axis radius `radius_y`, and `stops`, using
    /// the default [`ExtendMode::Pad`] extend mode. Equal radii yield a circle.
    #[must_use]
    pub fn new_elliptical(
        center: Point,
        radius_x: f32,
        radius_y: f32,
        stops: Vec<GradientStop>,
    ) -> Self {
        Self {
            center,
            radius: radius_x,
            radius_y,
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

    /// Returns the gradient's horizontal (x-axis) radius (offset `1.0`).
    #[must_use]
    pub const fn radius(&self) -> f32 {
        self.radius
    }

    /// Returns the gradient's vertical (y-axis) radius. Equal to [`radius`] for a
    /// circle; different for an ellipse.
    ///
    /// [`radius`]: Self::radius
    #[must_use]
    pub const fn radius_y(&self) -> f32 {
        self.radius_y
    }

    /// Whether the ending shape is an ellipse (the two radii differ).
    #[must_use]
    pub fn is_elliptical(&self) -> bool {
        self.radius_y != self.radius
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
        // A plain `new` is circular: both radii equal.
        assert_eq!(g.radius_y(), 10.0);
        assert!(!g.is_elliptical());
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.extend(), ExtendMode::Pad);
    }

    #[test]
    fn elliptical_carries_two_radii() {
        let g = RadialGradient::new_elliptical(Point::new(5.0, 5.0), 30.0, 10.0, stops());
        assert_eq!(g.radius(), 30.0);
        assert_eq!(g.radius_y(), 10.0);
        assert!(g.is_elliptical());
        // Equal radii collapse back to a circle.
        let c = RadialGradient::new_elliptical(Point::ORIGIN, 12.0, 12.0, stops());
        assert!(!c.is_elliptical());
    }

    #[test]
    fn with_extend_overrides() {
        let g = RadialGradient::new(Point::ORIGIN, 10.0, stops()).with_extend(ExtendMode::Reflect);
        assert_eq!(g.extend(), ExtendMode::Reflect);
    }
}
