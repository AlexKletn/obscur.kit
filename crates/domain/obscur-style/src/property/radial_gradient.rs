#![doc = include_str!("radial_gradient.doc/_module.md")]

use crate::property::{GradientStop, Length, RadialShape, RadialSize};

#[doc = include_str!("radial_gradient.doc/RadialGradient.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct RadialGradient {
    #[doc = include_str!("radial_gradient.doc/shape.md")]
    pub shape: RadialShape,
    #[doc = include_str!("radial_gradient.doc/size.md")]
    pub size: RadialSize,
    #[doc = include_str!("radial_gradient.doc/center_x.md")]
    pub center_x: Length,
    #[doc = include_str!("radial_gradient.doc/center_y.md")]
    pub center_y: Length,
    #[doc = include_str!("radial_gradient.doc/stops.md")]
    pub stops: Box<[GradientStop]>,
}

impl RadialGradient {
    #[doc = include_str!("radial_gradient.doc/new.md")]
    #[must_use]
    pub fn new(
        shape: RadialShape,
        size: RadialSize,
        center_x: Length,
        center_y: Length,
        stops: Vec<GradientStop>,
    ) -> Self {
        Self {
            shape,
            size,
            center_x,
            center_y,
            stops: stops.into_boxed_slice(),
        }
    }

    #[doc = include_str!("radial_gradient.doc/shape_2.md")]
    #[must_use]
    pub const fn shape(&self) -> RadialShape {
        self.shape
    }

    #[doc = include_str!("radial_gradient.doc/size_2.md")]
    #[must_use]
    pub const fn size(&self) -> RadialSize {
        self.size
    }

    #[doc = include_str!("radial_gradient.doc/center_x_2.md")]
    #[must_use]
    pub const fn center_x(&self) -> Length {
        self.center_x
    }

    #[doc = include_str!("radial_gradient.doc/center_y_2.md")]
    #[must_use]
    pub const fn center_y(&self) -> Length {
        self.center_y
    }

    #[doc = include_str!("radial_gradient.doc/uses_ch.md")]
    #[must_use]
    pub const fn uses_ch(&self) -> bool {
        self.center_x().is_ch()
            || self.center_y().is_ch()
            || match self.size() {
                RadialSize::Length(length) => length.is_ch(),
                _ => false,
            }
    }

    #[doc = include_str!("radial_gradient.doc/stops_2.md")]
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
        let g = RadialGradient::new(
            RadialShape::Circle,
            RadialSize::FarthestCorner,
            Length::Percent(30.0),
            Length::Percent(40.0),
            vec![
                GradientStop::new(StyleColor::BLACK, 0.0),
                GradientStop::new(StyleColor::rgba(1.0, 1.0, 1.0, 1.0), 1.0),
            ],
        );
        assert_eq!(g.shape(), RadialShape::Circle);
        assert_eq!(g.size(), RadialSize::FarthestCorner);
        assert_eq!(g.center_x(), Length::Percent(30.0));
        assert_eq!(g.center_y(), Length::Percent(40.0));
        assert_eq!(g.stops().len(), 2);
        assert_eq!(g.stops()[0].position(), Some(0.0));
    }
}
