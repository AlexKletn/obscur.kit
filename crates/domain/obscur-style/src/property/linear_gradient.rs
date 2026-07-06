//! A style-local `linear-gradient(...)` value (spec §5.2).

use crate::property::GradientStop;

/// A CSS `linear-gradient(...)` image, as authored in a stylesheet.
///
/// The gradient is described by an `angle_deg` on the CSS convention — `0` points
/// *to top*, `90` *to right*, `180` *to bottom*, `270` *to left*, increasing
/// clockwise — and an ordered list of color [`GradientStop`]s. The `to <side>`
/// keyword forms are folded to their equivalent angle by the CSS compiler, so
/// this type stores only an angle.
///
/// The angle defines the gradient *line*; the scene coordinator turns it plus the
/// painted box's bounds into the concrete start/end points the renderer's
/// `obscur_scene::paint::LinearGradient` needs. This type is a pure style concept
/// and deliberately has **no dependency on the scene**.
///
/// Because [`GradientStop`] carries `f32` data, this type derives [`PartialEq`]
/// but not [`Eq`]/[`Hash`].
#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    /// The gradient line's angle in degrees (CSS convention: `0` = to top,
    /// increasing clockwise).
    pub angle_deg: f32,
    /// The gradient's color stops, in authored order along the line.
    pub stops: Box<[GradientStop]>,
}

impl LinearGradient {
    /// Creates a linear gradient at `angle_deg` (CSS convention) with `stops`.
    #[must_use]
    pub fn new(angle_deg: f32, stops: Vec<GradientStop>) -> Self {
        Self {
            angle_deg,
            stops: stops.into_boxed_slice(),
        }
    }

    /// Returns the gradient line's angle in degrees (CSS convention).
    #[must_use]
    pub const fn angle_deg(&self) -> f32 {
        self.angle_deg
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
    }
}
