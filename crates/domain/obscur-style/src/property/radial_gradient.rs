//! A style-local `radial-gradient(...)` value (spec §5.2).

use crate::property::{GradientStop, Length, RadialShape, RadialSize};

/// A CSS `radial-gradient(...)` image, as authored in a stylesheet.
///
/// The gradient is described by an ending [`shape`](RadialShape) (circle or
/// ellipse), an ending [`size`](RadialSize) (an extent keyword or explicit
/// radius), a center position (`center_x` / `center_y`, box-relative
/// [`Length`]s — the `at <position>`), and an ordered list of color
/// [`GradientStop`]s. Every parameter is stored **box-relative** and resolved
/// against the painted box at paint time, mirroring how [`LinearGradient`] stores
/// only an angle: the scene coordinator turns the shape/size/center plus the
/// box's bounds into the concrete center and radius the renderer's
/// `obscur_scene::paint::RadialGradient` needs. This type is a pure style concept
/// and deliberately has **no dependency on the scene**.
///
/// [`LinearGradient`]: crate::property::LinearGradient
///
/// Because [`GradientStop`] carries `f32` data, this type derives [`PartialEq`]
/// but not [`Eq`]/[`Hash`].
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
        }
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

    /// Whether the center or an explicit radius is a `ch` term.
    ///
    /// Covers exactly the lengths `absolutize_radial` folds; the stop offsets are
    /// percentages, not lengths.
    #[must_use]
    pub const fn uses_ch(&self) -> bool {
        self.center_x().is_ch()
            || self.center_y().is_ch()
            || match self.size() {
                RadialSize::Length(length) => length.is_ch(),
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
