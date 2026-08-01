#![doc = include_str!("conic_gradient.doc/_module.md")]

use obscur_geometry::Point;

use super::{ExtendMode, GradientStop};

#[doc = include_str!("conic_gradient.doc/ConicGradient.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ConicGradient {
    #[doc = include_str!("conic_gradient.doc/center.md")]
    pub center: Point,
    #[doc = include_str!("conic_gradient.doc/start_angle.md")]
    pub start_angle: f32,
    #[doc = include_str!("conic_gradient.doc/stops.md")]
    pub stops: Vec<GradientStop>,
    #[doc = include_str!("conic_gradient.doc/extend.md")]
    pub extend: ExtendMode,
}

impl ConicGradient {
    #[doc = include_str!("conic_gradient.doc/new.md")]
    #[must_use]
    pub fn new(center: Point, start_angle: f32, stops: Vec<GradientStop>) -> Self {
        Self {
            center,
            start_angle,
            stops,
            extend: ExtendMode::Pad,
        }
    }

    #[doc = include_str!("conic_gradient.doc/with_extend.md")]
    #[must_use]
    pub fn with_extend(mut self, mode: ExtendMode) -> Self {
        self.extend = mode;
        self
    }

    #[doc = include_str!("conic_gradient.doc/center_2.md")]
    #[must_use]
    pub const fn center(&self) -> Point {
        self.center
    }

    #[doc = include_str!("conic_gradient.doc/start_angle_2.md")]
    #[must_use]
    pub const fn start_angle(&self) -> f32 {
        self.start_angle
    }

    #[doc = include_str!("conic_gradient.doc/stops_2.md")]
    #[must_use]
    pub fn stops(&self) -> &[GradientStop] {
        &self.stops
    }

    #[doc = include_str!("conic_gradient.doc/extend_2.md")]
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
