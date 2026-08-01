#![doc = include_str!("solid_brush.doc/_module.md")]

use crate::color::Color;

#[doc = include_str!("solid_brush.doc/SolidBrush.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolidBrush {
    #[doc = include_str!("solid_brush.doc/color.md")]
    pub color: Color,
}

impl SolidBrush {
    #[doc = include_str!("solid_brush.doc/new.md")]
    #[must_use]
    pub const fn new(color: Color) -> Self {
        Self { color }
    }

    #[doc = include_str!("solid_brush.doc/color_2.md")]
    #[must_use]
    pub const fn color(self) -> Color {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_color() {
        let b = SolidBrush::new(Color::WHITE);
        assert_eq!(b.color(), Color::WHITE);
    }
}
