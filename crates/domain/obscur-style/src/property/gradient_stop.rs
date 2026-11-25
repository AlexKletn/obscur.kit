//! A single color stop in a style-local CSS gradient.

use crate::property::StyleColor;

#[doc = include_str!("../../docs/api/property/gradient_stop.doc/GradientStop.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    /// The stop's color, as authored (straight-alpha sRGB).
    pub color: StyleColor,
    /// The stop's position along the gradient line in `0.0..=1.0`, or [`None`]
    /// when unauthored (auto-distributed).
    pub position: Option<f32>,
}

impl GradientStop {
    /// Creates a gradient stop with `color` at an explicit `position` (`0.0..=1.0`).
    #[must_use]
    pub const fn new(color: StyleColor, position: f32) -> Self {
        Self {
            color,
            position: Some(position),
        }
    }

    /// Creates a gradient stop with `color` and no authored position (its used
    /// position is auto-distributed).
    #[must_use]
    pub const fn auto(color: StyleColor) -> Self {
        Self {
            color,
            position: None,
        }
    }

    /// Returns the stop's color.
    #[must_use]
    pub const fn color(self) -> StyleColor {
        self.color
    }

    /// Returns the stop's authored position, if any.
    #[must_use]
    pub const fn position(self) -> Option<f32> {
        self.position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_color_and_position() {
        let s = GradientStop::new(StyleColor::BLACK, 0.25);
        assert_eq!(s.color(), StyleColor::BLACK);
        assert_eq!(s.position(), Some(0.25));
    }

    #[test]
    fn auto_has_no_position() {
        let s = GradientStop::auto(StyleColor::BLACK);
        assert_eq!(s.position(), None);
    }
}
