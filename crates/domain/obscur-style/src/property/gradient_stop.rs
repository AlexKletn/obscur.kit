#![doc = include_str!("gradient_stop.doc/_module.md")]

use crate::property::StyleColor;

#[doc = include_str!("gradient_stop.doc/GradientStop.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    #[doc = include_str!("gradient_stop.doc/color.md")]
    pub color: StyleColor,
    #[doc = include_str!("gradient_stop.doc/position.md")]
    pub position: Option<f32>,
}

impl GradientStop {
    #[doc = include_str!("gradient_stop.doc/new.md")]
    #[must_use]
    pub const fn new(color: StyleColor, position: f32) -> Self {
        Self {
            color,
            position: Some(position),
        }
    }

    #[doc = include_str!("gradient_stop.doc/auto.md")]
    #[must_use]
    pub const fn auto(color: StyleColor) -> Self {
        Self {
            color,
            position: None,
        }
    }

    #[doc = include_str!("gradient_stop.doc/color_2.md")]
    #[must_use]
    pub const fn color(self) -> StyleColor {
        self.color
    }

    #[doc = include_str!("gradient_stop.doc/position_2.md")]
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
