#![doc = include_str!("linear_gradient.doc/_module.md")]

use obscur_geometry::Point;

use super::{ExtendMode, GradientStop};

#[doc = include_str!("linear_gradient.doc/LinearGradient.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    #[doc = include_str!("linear_gradient.doc/start.md")]
    pub start: Point,
    #[doc = include_str!("linear_gradient.doc/end.md")]
    pub end: Point,
    #[doc = include_str!("linear_gradient.doc/stops.md")]
    pub stops: Vec<GradientStop>,
    #[doc = include_str!("linear_gradient.doc/extend.md")]
    pub extend: ExtendMode,
}

impl LinearGradient {
    #[doc = include_str!("linear_gradient.doc/new.md")]
    #[must_use]
    pub fn new(start: Point, end: Point, stops: Vec<GradientStop>) -> Self {
        Self {
            start,
            end,
            stops,
            extend: ExtendMode::Pad,
        }
    }

    #[doc = include_str!("linear_gradient.doc/with_extend.md")]
    #[must_use]
    pub fn with_extend(mut self, mode: ExtendMode) -> Self {
        self.extend = mode;
        self
    }

    #[doc = include_str!("linear_gradient.doc/start_2.md")]
    #[must_use]
    pub const fn start(&self) -> Point {
        self.start
    }

    #[doc = include_str!("linear_gradient.doc/end_2.md")]
    #[must_use]
    pub const fn end(&self) -> Point {
        self.end
    }

    #[doc = include_str!("linear_gradient.doc/stops_2.md")]
    #[must_use]
    pub fn stops(&self) -> &[GradientStop] {
        &self.stops
    }

    #[doc = include_str!("linear_gradient.doc/extend_2.md")]
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
