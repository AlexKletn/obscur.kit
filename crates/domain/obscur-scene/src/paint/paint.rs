//! How a shape is painted: filled or stroked.

use crate::color::Color;

use super::{Brush, Stroke};

#[doc = include_str!("../../docs/api/paint/paint.doc/Paint.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum Paint {
    /// Fill the shape's interior with a brush.
    Fill(Brush),
    /// Draw the shape's outline with a stroke.
    Stroke(Stroke),
}

impl Paint {
    /// Creates a fill paint using `brush`.
    #[must_use]
    pub const fn fill(brush: Brush) -> Self {
        Self::Fill(brush)
    }

    /// Creates a fill paint using a solid brush of `color`.
    #[must_use]
    pub const fn solid_fill(color: Color) -> Self {
        Self::Fill(Brush::solid(color))
    }

    /// Creates a stroke paint using `stroke`.
    #[must_use]
    pub const fn stroke(stroke: Stroke) -> Self {
        Self::Stroke(stroke)
    }

    /// Returns whether this is a [`Paint::Fill`].
    #[must_use]
    pub const fn is_fill(&self) -> bool {
        matches!(self, Self::Fill(_))
    }

    /// Returns whether this is a [`Paint::Stroke`].
    #[must_use]
    pub const fn is_stroke(&self) -> bool {
        matches!(self, Self::Stroke(_))
    }

    /// Returns the brush that colors this paint: the fill's brush, or the
    /// stroke's brush.
    #[must_use]
    pub const fn brush(&self) -> &Brush {
        match self {
            Self::Fill(brush) => brush,
            Self::Stroke(stroke) => stroke.brush(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_fill_is_fill_with_solid_brush() {
        let p = Paint::solid_fill(Color::WHITE);
        assert!(p.is_fill());
        assert!(!p.is_stroke());
        assert!(p.brush().is_solid());
    }

    #[test]
    fn stroke_paint_exposes_stroke_brush() {
        let p = Paint::stroke(Stroke::new(Brush::solid(Color::BLACK), 1.0));
        assert!(p.is_stroke());
        assert!(!p.is_fill());
        assert!(p.brush().is_solid());
    }

    #[test]
    fn fill_wraps_brush() {
        let p = Paint::fill(Brush::solid(Color::BLACK));
        assert!(p.is_fill());
    }
}
