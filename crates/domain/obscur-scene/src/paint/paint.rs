#![doc = include_str!("paint.doc/_module.md")]

use crate::color::Color;

use super::{Brush, Stroke};

#[doc = include_str!("paint.doc/Paint.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum Paint {
    #[doc = include_str!("paint.doc/Fill.md")]
    Fill(Brush),
    #[doc = include_str!("paint.doc/Stroke.md")]
    Stroke(Stroke),
}

impl Paint {
    #[doc = include_str!("paint.doc/fill.md")]
    #[must_use]
    pub const fn fill(brush: Brush) -> Self {
        Self::Fill(brush)
    }

    #[doc = include_str!("paint.doc/solid_fill.md")]
    #[must_use]
    pub const fn solid_fill(color: Color) -> Self {
        Self::Fill(Brush::solid(color))
    }

    #[doc = include_str!("paint.doc/stroke.md")]
    #[must_use]
    pub const fn stroke(stroke: Stroke) -> Self {
        Self::Stroke(stroke)
    }

    #[doc = include_str!("paint.doc/is_fill.md")]
    #[must_use]
    pub const fn is_fill(&self) -> bool {
        matches!(self, Self::Fill(_))
    }

    #[doc = include_str!("paint.doc/is_stroke.md")]
    #[must_use]
    pub const fn is_stroke(&self) -> bool {
        matches!(self, Self::Stroke(_))
    }

    #[doc = include_str!("paint.doc/brush.md")]
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
