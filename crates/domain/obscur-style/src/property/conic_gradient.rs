#![doc = include_str!("conic_gradient.doc/_module.md")]

use crate::property::{GradientStop, Length};

#[doc = include_str!("conic_gradient.doc/ConicGradient.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ConicGradient {
    #[doc = include_str!("conic_gradient.doc/from_angle_deg.md")]
    pub from_angle_deg: f32,
    #[doc = include_str!("conic_gradient.doc/center_x.md")]
    pub center_x: Length,
    #[doc = include_str!("conic_gradient.doc/center_y.md")]
    pub center_y: Length,
    #[doc = include_str!("conic_gradient.doc/stops.md")]
    pub stops: Box<[GradientStop]>,
}

impl ConicGradient {
    #[doc = include_str!("conic_gradient.doc/new.md")]
    #[must_use]
    pub fn new(
        from_angle_deg: f32,
        center_x: Length,
        center_y: Length,
        stops: Vec<GradientStop>,
    ) -> Self {
        Self {
            from_angle_deg,
            center_x,
            center_y,
            stops: stops.into_boxed_slice(),
        }
    }

    #[doc = include_str!("conic_gradient.doc/from_angle_deg_2.md")]
    #[must_use]
    pub const fn from_angle_deg(&self) -> f32 {
        self.from_angle_deg
    }

    #[doc = include_str!("conic_gradient.doc/center_x_2.md")]
    #[must_use]
    pub const fn center_x(&self) -> Length {
        self.center_x
    }

    #[doc = include_str!("conic_gradient.doc/center_y_2.md")]
    #[must_use]
    pub const fn center_y(&self) -> Length {
        self.center_y
    }

    #[doc = include_str!("conic_gradient.doc/uses_ch.md")]
    #[must_use]
    pub const fn uses_ch(&self) -> bool {
        self.center_x().is_ch() || self.center_y().is_ch()
    }

    #[doc = include_str!("conic_gradient.doc/stops_2.md")]
    #[must_use]
    pub const fn stops(&self) -> &[GradientStop] {
        &self.stops
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::StyleColor;

    #[test]
    fn new_stores_all_parameters() {
        let g = ConicGradient::new(
            90.0,
            Length::Percent(50.0),
            Length::Percent(50.0),
            vec![
                GradientStop::new(StyleColor::BLACK, 0.0),
                GradientStop::new(StyleColor::rgba(1.0, 1.0, 1.0, 1.0), 1.0),
            ],
        );
        assert_eq!(g.from_angle_deg(), 90.0);
        assert_eq!(g.center_x(), Length::Percent(50.0));
        assert_eq!(g.center_y(), Length::Percent(50.0));
        assert_eq!(g.stops().len(), 2);
    }
}
