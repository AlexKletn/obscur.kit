#![doc = include_str!("linear_gradient.doc/_module.md")]

use crate::property::{GradientStop, Length};

#[doc = include_str!("linear_gradient.doc/Corner.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    #[doc = include_str!("linear_gradient.doc/TopLeft.md")]
    TopLeft,
    #[doc = include_str!("linear_gradient.doc/TopRight.md")]
    TopRight,
    #[doc = include_str!("linear_gradient.doc/BottomRight.md")]
    BottomRight,
    #[doc = include_str!("linear_gradient.doc/BottomLeft.md")]
    BottomLeft,
}

#[doc = include_str!("linear_gradient.doc/LinearGradient.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    #[doc = include_str!("linear_gradient.doc/angle_deg.md")]
    pub angle_deg: f32,
    #[doc = include_str!("linear_gradient.doc/corner.md")]
    pub corner: Option<Corner>,
    #[doc = include_str!("linear_gradient.doc/stops.md")]
    pub stops: Box<[GradientStop]>,
    #[doc = include_str!("linear_gradient.doc/repeating.md")]
    pub repeating: bool,
    #[doc = include_str!("linear_gradient.doc/period.md")]
    pub period: Option<Length>,
}

impl LinearGradient {
    #[doc = include_str!("linear_gradient.doc/new.md")]
    #[must_use]
    pub fn new(angle_deg: f32, stops: Vec<GradientStop>) -> Self {
        Self {
            angle_deg,
            corner: None,
            stops: stops.into_boxed_slice(),
            repeating: false,
            period: None,
        }
    }

    #[doc = include_str!("linear_gradient.doc/new_corner.md")]
    #[must_use]
    pub fn new_corner(corner: Corner, angle_deg: f32, stops: Vec<GradientStop>) -> Self {
        Self {
            angle_deg,
            corner: Some(corner),
            stops: stops.into_boxed_slice(),
            repeating: false,
            period: None,
        }
    }

    #[doc = include_str!("linear_gradient.doc/with_corner.md")]
    #[must_use]
    pub fn with_corner(mut self, corner: Corner) -> Self {
        self.corner = Some(corner);
        self
    }

    #[doc = include_str!("linear_gradient.doc/new_repeating.md")]
    #[must_use]
    pub fn new_repeating(angle_deg: f32, stops: Vec<GradientStop>, period: Length) -> Self {
        Self {
            angle_deg,
            corner: None,
            stops: stops.into_boxed_slice(),
            repeating: true,
            period: Some(period),
        }
    }

    #[doc = include_str!("linear_gradient.doc/angle_deg_2.md")]
    #[must_use]
    pub const fn angle_deg(&self) -> f32 {
        self.angle_deg
    }

    #[doc = include_str!("linear_gradient.doc/corner_2.md")]
    #[must_use]
    pub const fn corner(&self) -> Option<Corner> {
        self.corner
    }

    #[doc = include_str!("linear_gradient.doc/stops_2.md")]
    #[must_use]
    pub const fn stops(&self) -> &[GradientStop] {
        &self.stops
    }

    #[doc = include_str!("linear_gradient.doc/repeating_2.md")]
    #[must_use]
    pub const fn repeating(&self) -> bool {
        self.repeating
    }

    #[doc = include_str!("linear_gradient.doc/period_2.md")]
    #[must_use]
    pub const fn period(&self) -> Option<Length> {
        self.period
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::StyleColor;

    #[test]
    fn new_stores_angle_and_stops() {
        let g = LinearGradient::new(
            90.0,
            vec![
                GradientStop::new(StyleColor::BLACK, 0.0),
                GradientStop::new(StyleColor::rgba(1.0, 1.0, 1.0, 1.0), 1.0),
            ],
        );
        assert_eq!(g.angle_deg(), 90.0);
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.stops()[0].position(), Some(0.0));
        assert_eq!(g.stops()[1].color(), StyleColor::rgba(1.0, 1.0, 1.0, 1.0));
        // A plain gradient is non-repeating and carries no period.
        assert!(!g.repeating());
        assert_eq!(g.period(), None);
    }

    #[test]
    fn repeating_carries_period_and_flag() {
        let g = LinearGradient::new_repeating(
            90.0,
            vec![
                GradientStop::new(StyleColor::BLACK, 0.0),
                GradientStop::new(StyleColor::rgba(1.0, 1.0, 1.0, 1.0), 1.0),
            ],
            Length::px(8.0),
        );
        assert!(g.repeating());
        assert_eq!(g.period(), Some(Length::px(8.0)));
    }
}
