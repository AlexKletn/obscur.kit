//! A style-local `radial-gradient(...)` value (spec §5.2).

use crate::property::{GradientStop, Length, RadialShape, RadialSize};

#[doc = include_str!("../../docs/api/property/radial_gradient.doc/RadialGradient.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct RadialGradient {
    /// The ending shape (circle or ellipse).
    pub shape: RadialShape,
    /// The ending size (an extent keyword or an explicit radius).
    pub size: RadialSize,
    /// The center's horizontal position, box-relative (default `50%`).
    pub center_x: Length,
    /// The center's vertical position, box-relative (default `50%`).
    pub center_y: Length,
    /// The gradient's color stops, in authored order outward from the center.
    pub stops: Box<[GradientStop]>,
    /// Length and percentage stop positions retained until the gradient radius is known.
    repeat_positions: Option<Box<[Option<Length>]>>,
}

impl RadialGradient {
    /// Creates a radial gradient with the given ending shape/size, center
    /// position, and stops.
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
            repeat_positions: None,
        }
    }

    /// Associates repeat positions with the gradient stops when there is one position per stop.
    #[must_use]
    pub fn with_repeat_positions(mut self, positions: Box<[Option<Length>]>) -> Self {
        if positions.len() == self.stops.len() && positions.len() >= 2 {
            self.repeat_positions = Some(positions);
        }
        self
    }

    /// Returns the authored positions when the gradient repeats.
    #[must_use]
    pub fn repeat_positions(&self) -> Option<&[Option<Length>]> {
        self.repeat_positions
            .as_deref()
            .filter(|positions| positions.len() == self.stops.len())
    }

    /// Returns the ending shape.
    #[must_use]
    pub const fn shape(&self) -> RadialShape {
        self.shape
    }

    /// Returns the ending size.
    #[must_use]
    pub const fn size(&self) -> RadialSize {
        self.size
    }

    /// Returns the center's horizontal position (box-relative).
    #[must_use]
    pub const fn center_x(&self) -> Length {
        self.center_x
    }

    /// Returns the center's vertical position (box-relative).
    #[must_use]
    pub const fn center_y(&self) -> Length {
        self.center_y
    }

    #[doc = include_str!("../../docs/api/property/radial_gradient.doc/uses_ch.md")]
    #[must_use]
    pub fn uses_ch(&self) -> bool {
        self.center_x().is_ch()
            || self.center_y().is_ch()
            || self
                .repeat_positions()
                .is_some_and(|positions| positions.iter().flatten().any(|length| length.is_ch()))
            || match self.size() {
                RadialSize::Length(length) => length.is_ch(),
                RadialSize::Ellipse(x, y) => x.is_ch() || y.is_ch(),
                _ => false,
            }
    }

    /// Returns the gradient's color stops.
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
