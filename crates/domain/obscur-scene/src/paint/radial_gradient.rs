#![doc = include_str!("radial_gradient.doc/_module.md")]

use obscur_geometry::Point;

use super::{ExtendMode, GradientStop};

#[doc = include_str!("radial_gradient.doc/RadialGradient.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct RadialGradient {
    #[doc = include_str!("radial_gradient.doc/center.md")]
    pub center: Point,
    #[doc = include_str!("radial_gradient.doc/radius.md")]
    pub radius: f32,
    #[doc = include_str!("radial_gradient.doc/radius_y.md")]
    pub radius_y: f32,
    #[doc = include_str!("radial_gradient.doc/stops.md")]
    pub stops: Vec<GradientStop>,
    #[doc = include_str!("radial_gradient.doc/extend.md")]
    pub extend: ExtendMode,
}

impl RadialGradient {
    #[doc = include_str!("radial_gradient.doc/new.md")]
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

    #[doc = include_str!("radial_gradient.doc/new_elliptical.md")]
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

    #[doc = include_str!("radial_gradient.doc/with_extend.md")]
    #[must_use]
    pub fn with_extend(mut self, mode: ExtendMode) -> Self {
        self.extend = mode;
        self
    }

    #[doc = include_str!("radial_gradient.doc/center_2.md")]
    #[must_use]
    pub const fn center(&self) -> Point {
        self.center
    }

    #[doc = include_str!("radial_gradient.doc/radius_2.md")]
    #[must_use]
    pub const fn radius(&self) -> f32 {
        self.radius
    }

    #[doc = include_str!("radial_gradient.doc/radius_y_2.md")]
    #[must_use]
    pub const fn radius_y(&self) -> f32 {
        self.radius_y
    }

    #[doc = include_str!("radial_gradient.doc/is_elliptical.md")]
    #[must_use]
    pub fn is_elliptical(&self) -> bool {
        self.radius_y != self.radius
    }

    #[doc = include_str!("radial_gradient.doc/stops_2.md")]
    #[must_use]
    pub fn stops(&self) -> &[GradientStop] {
        &self.stops
    }

    #[doc = include_str!("radial_gradient.doc/extend_2.md")]
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
