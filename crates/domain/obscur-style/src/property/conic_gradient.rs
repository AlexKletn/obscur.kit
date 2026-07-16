//! A style-local `conic-gradient(...)` value (spec §5.2).

use crate::property::{GradientStop, Length};

/// A CSS `conic-gradient(...)` image, as authored in a stylesheet.
///
/// The gradient is described by a starting angle `from_angle_deg` on the CSS
/// convention — `0` points *up*, increasing **clockwise** — a center position
/// (`center_x` / `center_y`, box-relative [`Length`]s — the `at <position>`),
/// and an ordered list of color [`GradientStop`]s. Every parameter is stored
/// **box-relative** and resolved against the painted box at paint time,
/// mirroring how [`LinearGradient`] stores only an angle: the scene coordinator
/// turns the center plus the box's bounds into the concrete center, and the
/// starting angle into the scene sweep's `start_angle`, that the renderer's
/// `obscur_scene::paint::ConicGradient` needs. This type is a pure style concept
/// and deliberately has **no dependency on the scene**.
///
/// [`LinearGradient`]: crate::property::LinearGradient
///
/// Because [`GradientStop`] carries `f32` data, this type derives [`PartialEq`]
/// but not [`Eq`]/[`Hash`].
#[derive(Debug, Clone, PartialEq)]
pub struct ConicGradient {
    /// The starting angle in degrees (CSS convention: `0` = up, increasing
    /// clockwise); the default is `0`.
    pub from_angle_deg: f32,
    /// The center's horizontal position, box-relative (default `50%`).
    pub center_x: Length,
    /// The center's vertical position, box-relative (default `50%`).
    pub center_y: Length,
    /// The gradient's color stops, in authored order around the sweep.
    pub stops: Box<[GradientStop]>,
}

impl ConicGradient {
    /// Creates a conic gradient with the given starting angle, center position,
    /// and stops.
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

    /// Returns the starting angle in degrees (CSS convention).
    #[must_use]
    pub const fn from_angle_deg(&self) -> f32 {
        self.from_angle_deg
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

    /// Whether the center is a `ch` term.
    ///
    /// Covers exactly the lengths `absolutize_conic` folds; the start angle is an
    /// angle and the stop offsets are percentages.
    #[must_use]
    pub const fn uses_ch(&self) -> bool {
        self.center_x().is_ch() || self.center_y().is_ch()
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
