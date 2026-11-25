//! A flat, single-color brush.

use crate::color::Color;

/// A brush that paints a shape with a single flat [`Color`] (spec §5.1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolidBrush {
    /// The fill color.
    pub color: Color,
}

impl SolidBrush {
    /// Creates a solid brush that paints with `color`.
    #[must_use]
    pub const fn new(color: Color) -> Self {
        Self { color }
    }

    /// Returns the brush's color.
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
